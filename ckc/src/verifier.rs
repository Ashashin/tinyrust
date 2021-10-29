use crate::prover::{run_instrumented_vm, validate_hash, Proof, ProofStrategy};
use crate::stats;

pub struct Verifier {}
pub struct ProofReport {
    pub proof: Proof,
    pub eta: f64,
    pub q: f64,
    pub valid: bool,
}

#[derive(PartialEq, Eq, Debug)]
enum ValidationResult {
    IncorrectHash,
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
        let claim = format!("Claim: all values in {:?}", self.proof.params.input_domain);
        let proof_eta = format!("Probability to find this proof: {}", self.eta);
        let proof_q = format!("Probability that claim is true: {}", self.q);

        let report = [proof_strategy, claim, proof_eta, proof_q].join("\n\t");
        let report = format!("REPORT for {}\n\t{}\n\n{}", program, report, proof_valid);

        println!("{}", report);
    }
}

impl Verifier {
    pub fn check_proof(proof: Proof, epsilon: f64) -> ProofReport {
        match proof.params.strategy {
            ProofStrategy::FixedEffort => Self::check_proof_fixed_effort(proof, epsilon),
            ProofStrategy::BestEffort => Self::check_proof_best_effort(proof),
            ProofStrategy::OverTesting => Self::check_proof_overtesting(proof),

            _ => unimplemented!("Unsupported proof strategy: {:?}", proof.params.strategy),
        }
    }

    fn check_proof_fixed_effort(proof: Proof, epsilon: f64) -> ProofReport {
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.params.v;
        let eta = stats::compute_eta(kappa, u, v);
        let q = stats::compute_q(kappa, u, v);

        let valid = q > 1.0 - epsilon
            && Self::validate_vset(&proof) == ValidationResult::Valid;

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
            Self::validate_vset(&proof),
            ValidationResult::Valid | ValidationResult::ValidButTooFewHashes(_)
        );

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

        let valid = matches!(
            Self::validate_vset(&proof),
            ValidationResult::Valid
        );

        ProofReport {
            proof,
            eta,
            q,
            valid,
        }
    }

    fn validate_vset(proof: &Proof) -> ValidationResult {
        let enough_hashes = proof.vset.len() >= proof.params.v;

        for &i in proof.vset.as_slice() {
            if !proof.domain.contains(&i) {
                // Blocking error
                return ValidationResult::IncorrectInput(i);
            }

            match run_instrumented_vm(proof.params.program_file.clone(), i) {
                Ok(res) => {
                    if res.output != proof.params.expected_output {
                        return ValidationResult::IncorrectOutput(res.output);
                    }

                    if !validate_hash(res.hash, proof.params.kappa as usize) {
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
