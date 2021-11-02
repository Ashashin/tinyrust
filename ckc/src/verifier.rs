use std::{ops::Range, time::Instant};

use crate::{
    proof::{Proof, ProofReport, ProofStrategy},
    stats::{compute_eta, compute_q},
    vm::{validate_hash, InstrumentedVM},
};

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

pub struct Verifier {
    proof: Proof,
}

impl Verifier {
    pub fn new(proof: Proof) -> Self {
        Self { proof }
    }

    pub fn check_proof(&self, epsilon: f64) -> ProofReport {
        let start = Instant::now();
        let result = match self.proof.params.strategy {
            ProofStrategy::FixedEffort => Self::check_proof_fixed_effort(&self.proof, epsilon),
            ProofStrategy::BestEffort => Self::check_proof_best_effort(&self.proof),
            ProofStrategy::BestEffortAdaptive(_eta0) => Self::check_proof_best_effort(&self.proof),
            ProofStrategy::OverTesting(_eta0) => Self::check_proof_overtesting(&self.proof),
        };

        let duration = start.elapsed();

        println!("Verifier time: {:?}", duration);

        result
    }

    fn check_proof_fixed_effort(proof: &Proof, epsilon: f64) -> ProofReport {
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.params.v;
        let eta = compute_eta(kappa, u, v);
        let q = compute_q(kappa, u, v);

        let valid = !q.is_nan()
            && !eta.is_nan()
            && q > 1.0 - epsilon
            && Self::validate_vset(proof, &proof.params.input_domain) == ValidationResult::Valid;

        ProofReport::create(proof, eta, q, valid)
    }

    fn check_proof_best_effort(proof: &Proof) -> ProofReport {
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.vset.len();
        let eta = compute_eta(kappa, u, v);
        let q = compute_q(kappa, u, v);

        let valid = matches!(
            Self::validate_vset(proof, &proof.params.input_domain),
            ValidationResult::Valid | ValidationResult::ValidButTooFewHashes(_)
        ) && !q.is_nan()
            && !eta.is_nan();

        ProofReport::create(proof, eta, q, valid)
    }

    fn check_proof_overtesting(proof: &Proof) -> ProofReport {
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.vset.len();
        let eta = compute_eta(kappa, u, v);
        let q = compute_q(kappa, u, v);

        let domain = match proof.extended_domain {
            Some(ref extended) => extended,
            _ => &proof.params.input_domain,
        };

        let valid = matches!(Self::validate_vset(proof, domain), ValidationResult::Valid);

        ProofReport::create(proof, eta, q, valid)
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
