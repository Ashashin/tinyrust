use std::{ops::Range, time::Instant};

use crate::{
    proof::{Proof, ProofReport, ProofStrategy},
    stats::{compute_eta, compute_q},
    vm::{validate_hash, InstrumentedVM},
};

/// Enum of the possible outcome of the verification of the witnesses
#[derive(PartialEq, Eq, Debug)]
enum ValidationResult {
    /// An incorrect hash was found in the given witnesses set
    IncorrectHash,
    /// Program is not valid
    InvalidProgram,
    /// Witness given is outside the agreed domain
    IncorrectInput(usize),
    /// Program does not give exoected result
    IncorrectOutput(usize),
    /// Runtime Error of the program
    ExecutionError,
    /// No error but the number of witness if not enough
    ValidButTooFewHashes(usize),
    /// Valid witnesses set
    Valid,
}

/// Verifier
pub struct Verifier {
    /// Proof being verified
    proof: Proof,
}

impl Verifier {
    /// Create new verifier
    pub const fn new(proof: Proof) -> Self {
        Self { proof }
    }

    /// Validate proof
    pub fn check_proof(&self, epsilon: f64) -> ProofReport {
        let start = Instant::now();
        let result = match self.proof.params.strategy {
            ProofStrategy::FixedEffort => self.check_proof_fixed_effort(epsilon),
            ProofStrategy::BestEffort => self.check_proof_best_effort(),
            ProofStrategy::BestEffortAdaptive(_eta0) => self.check_proof_best_effort(),
            ProofStrategy::OverTesting(_eta0) => self.check_proof_overtesting(),
        };

        let duration = start.elapsed();

        println!("Verifier time: {:?}", duration);

        result
    }

    /// Validation for fixed effort
    fn check_proof_fixed_effort(&self, epsilon: f64) -> ProofReport {
        let proof = &self.proof;
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.params.v;
        let eta = compute_eta(kappa, u, v);
        let q = compute_q(kappa, u, v);

        let valid = !q.is_nan()
            && !eta.is_nan()
            && q > 1.0 - epsilon
            && self.validate_vset(&proof.params.input_domain) == ValidationResult::Valid;

        ProofReport::create(proof, eta, q, valid)
    }

    /// Validation for best effort
    fn check_proof_best_effort(&self) -> ProofReport {
        let proof = &self.proof;
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.vset.len();
        let eta = compute_eta(kappa, u, v);
        let q = compute_q(kappa, u, v);

        let valid = matches!(
            self.validate_vset(&proof.params.input_domain),
            ValidationResult::Valid | ValidationResult::ValidButTooFewHashes(_)
        ) && !q.is_nan()
            && !eta.is_nan();

        ProofReport::create(proof, eta, q, valid)
    }

    /// Validation for overtesting
    fn check_proof_overtesting(&self) -> ProofReport {
        let proof = &self.proof;
        let u = proof.params.input_domain.end - proof.params.input_domain.start;
        let kappa = proof.params.kappa;

        let v = proof.vset.len();
        let eta = compute_eta(kappa, u, v);
        let q = compute_q(kappa, u, v);

        let domain = match proof.extended_domain {
            Some(ref extended) => extended,
            _ => &proof.params.input_domain,
        };

        let valid = matches!(self.validate_vset(domain), ValidationResult::Valid);

        ProofReport::create(proof, eta, q, valid)
    }

    /// Validating the witness set
    fn validate_vset(&self, domain: &Range<usize>) -> ValidationResult {
        let proof = &self.proof;

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

                    if !validate_hash(&res.hash, proof.params.kappa as usize) {
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
