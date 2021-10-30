use crate::prover::{validate_hash, InstrumentedVM, Proof, ProofStrategy};
use crate::stats;

use std::{ops::Range, time::Instant};

use serde::Serialize;

pub struct Verifier {}

#[derive(Serialize)]
pub struct ProofReport {
    pub proof: Proof,
    pub eta: f64,
    pub q: f64,
    pub valid: bool,
}

#[derive(PartialEq, Eq, Debug)]
enum ValidationResult {
    IncorrectHash,
    InvalidProgram,
    IncorrectInput(usize),
    IncorrectOutput(usize),
    ExecutionError,
    ValidButTooFewHashes(usize),
    Valid,
}

impl ProofReport {
    pub fn display(&self) {
        let program = &self.proof.params.program_file;
        let proof_strategy = format!("Proof strategy: {:?}", self.proof.params.strategy);
        let proof_valid = format!("Proof is accepted: *{}*", self.valid);
        let proof_contents = format!("Witnesses: {}", self.proof.vset.len());
        let request = format!(
            "Request: all values in {:?}",
            self.proof.params.input_domain
        );

        let actual_domain = match self.proof.extended_domain {
            Some(ref extended) => extended,
            _ => &self.proof.params.input_domain,
        };

        let claim = format!("Claim: all values in {:?}", actual_domain);
        let proof_eta = format!("Probability to find this proof: {}", self.eta);
        let proof_q = format!("Probability that claim is true: {}", self.q);

        let report = [
            proof_strategy,
            request,
            claim,
            proof_contents,
            proof_eta,
            proof_q,
        ]
        .join("\n\t");
        let report = format!("REPORT for {}\n\t{}\n\t{}", program, report, proof_valid);

        println!("{}", report);
    }

    pub fn export(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl Verifier {
    pub fn check_proof(proof: Proof, epsilon: f64) -> ProofReport {
        let start = Instant::now();
        let result = match proof.params.strategy {
            ProofStrategy::FixedEffort => Self::check_proof_fixed_effort(proof, epsilon),
            ProofStrategy::BestEffort => Self::check_proof_best_effort(proof),
            ProofStrategy::BestEffortAdaptive(_eta0) => Self::check_proof_best_effort(proof),
            ProofStrategy::OverTesting(_eta0) => Self::check_proof_overtesting(proof),
        };

        let duration = start.elapsed();

        println!("Verifier time: {:?}", duration);

        result
    }

    fn check_proof_fixed_effort(proof: Proof, epsilon: f64) -> ProofReport {
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.params.v;
        let eta = stats::compute_eta(kappa, u, v);
        let q = stats::compute_q(kappa, u, v);

        let valid = !q.is_nan()
            && !eta.is_nan()
            && q > 1.0 - epsilon
            && Self::validate_vset(&proof, &proof.params.input_domain) == ValidationResult::Valid;

        ProofReport {
            proof,
            eta,
            q,
            valid,
        }
    }

    fn check_proof_best_effort(proof: Proof) -> ProofReport {
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.vset.len();
        let eta = stats::compute_eta(kappa, u, v);
        let q = stats::compute_q(kappa, u, v);

        let valid = matches!(
            Self::validate_vset(&proof, &proof.params.input_domain),
            ValidationResult::Valid | ValidationResult::ValidButTooFewHashes(_)
        ) && !q.is_nan()
            && !eta.is_nan();

        ProofReport {
            proof,
            eta,
            q,
            valid,
        }
    }

    fn check_proof_overtesting(proof: Proof) -> ProofReport {
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.vset.len();
        let eta = stats::compute_eta(kappa, u, v);
        let q = stats::compute_q(kappa, u, v);

        let domain = match proof.extended_domain {
            Some(ref extended) => extended,
            _ => &proof.params.input_domain,
        };

        let valid = matches!(Self::validate_vset(&proof, domain), ValidationResult::Valid);

        ProofReport {
            proof,
            eta,
            q,
            valid,
        }
    }

    fn validate_vset(proof: &Proof, domain: &Range<usize>) -> ValidationResult {
        let enough_hashes = proof.vset.len() >= proof.params.v;

        let mut vm = match InstrumentedVM::new(&proof.params.program_file) {
            Ok(ivm) => ivm,
            _ => return ValidationResult::InvalidProgram,
        };

        for &i in proof.vset.as_slice() {
            if !domain.contains(&i) {
                // Value is outside of authorised domain
                return ValidationResult::IncorrectInput(i);
            }

            match vm.run(i) {
                Ok(res) => {
                    if res.output != proof.params.expected_output {
                        // Output does not match expectation
                        return ValidationResult::IncorrectOutput(res.output);
                    }

                    if !validate_hash(res.hash, proof.params.kappa as usize) {
                        // Hash does not match expectation
                        return ValidationResult::IncorrectHash;
                    }
                }
                Err(_e) => return ValidationResult::ExecutionError,
            }
        }

        if enough_hashes {
            ValidationResult::Valid
        } else {
            ValidationResult::ValidButTooFewHashes(proof.vset.len())
        }
    }
}
