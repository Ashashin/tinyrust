use serde::{Deserialize, Serialize};

use std::ops::Range;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ProofStrategy {
    FixedEffort,
    BestEffort,
    BestEffortAdaptive(f64),
    OverTesting(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofParams {
    pub program_file: String,
    pub input_domain: Range<usize>,
    pub expected_output: usize,
    pub kappa: u64,
    pub v: usize,
    pub strategy: ProofStrategy,
}

impl ProofParams {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub vset: Vec<usize>,
    pub extended_domain: Option<Range<usize>>,
    pub params: ProofParams,
}

#[derive(Serialize)]
pub struct ProofReport {
    proof: Proof,
    eta: f64,
    q: f64,
    valid: bool,
}

impl ProofReport {
    pub fn create(proof: &Proof, eta: f64, q: f64, valid: bool) -> Self {
        Self {
            proof: proof.clone(),
            eta,
            q,
            valid,
        }
    }

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

        println!("JSON: {}", fake_proof.export());
    }
}
