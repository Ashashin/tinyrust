use serde::{Deserialize, Serialize};

use std::ops::Range;

/// Enum representing the available strategies
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ProofStrategy {
    /// Fixed Effort: Verifier check if a specific threshold is obtained
    FixedEffort(f64),
    /// Best Effort: Prover gives everything he can
    BestEffort,
    /// Best Effort Adaptive: Prover gives enough to obtain an acceptable proof
    BestEffortAdaptive(f64),
    /// Overtesting: Proves goes beyond the claim to get enough valid samples
    OverTesting(f64),
}

/// Parameters used for the proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofParams {
    /// The program used for the proof
    pub program_file: String,
    /// The testing domain of the claim
    pub input_domain: Range<usize>,
    /// The expected output of the program
    pub expected_output: usize,
    /// The agreed upon hash max value
    pub kappa: u64,
    /// The agreed upon number of witnesses
    pub v: usize,
    /// The proof strategy
    pub strategy: ProofStrategy,
}

impl ProofParams {
    /// Generate new params
    pub fn new(
        filename: &str,
        input_domain: Range<usize>,
        output: usize,
        kappa: u64,
        v: usize,
        strategy: ProofStrategy,
    ) -> Self {
        Self {
            program_file: String::from(filename),
            input_domain,
            expected_output: output,
            kappa,
            v,
            strategy,
        }
    }
}

/// Struct representing the proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// Witness set
    pub vset: Vec<usize>,
    /// Extended domain (for overting strategy)
    pub extended_domain: Option<Range<usize>>,
    /// Parameters of the proof
    pub params: ProofParams,
}

/// Report of the validity of the proof
#[derive(Serialize)]
pub struct ProofReport {
    /// The proof being reported
    pub proof: Proof,
    /// The probability of getting an acceptable proof
    pub eta: f64,
    /// The probability of the proof being valid
    pub q: f64,
    /// The conclusion of the report on whether the proof should be accepted
    pub valid: bool,
}

impl ProofReport {
    /// Create a new report
    pub fn create(proof: &Proof, eta: f64, q: f64, valid: bool) -> Self {
        Self {
            proof: proof.clone(),
            eta,
            q,
            valid,
        }
    }

    /// Print the report
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

    /// Export the report as the json
    pub fn export(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_display() {
        let fake_proof = ProofReport {
            proof: Proof {
                vset: vec![],
                params: ProofParams {
                    program_file: String::from("none.txt"),
                    input_domain: 42..69,
                    expected_output: 33,
                    kappa: 12,
                    v: 3,
                    strategy: ProofStrategy::BestEffortAdaptive(0.99),
                },
                extended_domain: None,
            },
            eta: 0.4,
            q: 0.6,
            valid: false,
        };

        fake_proof.display();
    }
}
