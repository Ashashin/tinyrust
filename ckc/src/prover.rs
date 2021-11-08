use color_eyre::Report;

use std::time::Instant;

use crate::{
    proof::{Proof, ProofParams, ProofStrategy},
    stats::{compute_delta_u, compute_v_min},
    vm::{validate_hash, InstrumentedVM, RunResult},
};

/// Prover
pub struct Prover {
    /// Params used for the validation of the proof
    params: ProofParams,
}

impl Prover {
    /// Create the Prover
    pub fn new(params: ProofParams) -> Self {
        assert!(params.kappa < 160);
        Self { params }
    }

    /// Get a proof for the claim
    pub fn obtain_proof(self) -> Result<Proof, Report> {
        let start = Instant::now();
        let result = match self.params.strategy {
            ProofStrategy::BestEffort => self.obtain_proof_best_effort(),
            ProofStrategy::FixedEffort(_epsilon) => self.obtain_proof_fixed_effort(),
            ProofStrategy::OverTesting(eta0) => self.obtain_proof_overtesting(eta0),
            ProofStrategy::BestEffortAdaptive(eta0) => self.obtain_proof_bea(eta0),
        };
        let duration = start.elapsed();

        println!("Prover time: {:?}", duration);

        result
    }

    /// Proof for best effort adaptive
    fn obtain_proof_bea(self, eta0: f64) -> Result<Proof, Report> {
        let u = self.params.input_domain.end - self.params.input_domain.start;
        let threshold = compute_v_min(eta0, self.params.kappa, u);

        let mut vset = vec![];
        let mut vm = InstrumentedVM::new(&self.params.program_file)?;

        for i in self.params.input_domain.clone() {
            let run_result = vm.run(i).unwrap();
            if self.select_witness(&run_result) {
                vset.push(i);
            }
            if vset.len() > threshold {
                break;
            }
        }

        Ok(Proof {
            vset,
            extended_domain: None,
            params: self.params,
        })
    }

    /// Proof for fixed effort
    fn obtain_proof_fixed_effort(self) -> Result<Proof, Report> {
        self.obtain_proof_best_effort()
    }

    /// Proof for best effort
    fn obtain_proof_best_effort(self) -> Result<Proof, Report> {
        let mut vset = vec![];
        let domain = self.params.input_domain.clone();

        let mut vm = InstrumentedVM::new(&self.params.program_file)?;

        domain.for_each(|i| {
            let run_result = vm.run(i).unwrap();
            if self.select_witness(&run_result) {
                vset.push(i);
            }
        });

        Ok(Proof {
            vset,
            extended_domain: None,
            params: self.params,
        })
    }

    /// Proof for overtesting
    fn obtain_proof_overtesting(self, eta0: f64) -> Result<Proof, Report> {
        let start = self.params.input_domain.start;
        let end = self.params.input_domain.end;

        let delta = compute_delta_u(eta0, self.params.kappa, end - start, self.params.v);
        let extended_domain = start..(end + delta);

        let mut vset = vec![];

        let mut vm = InstrumentedVM::new(&self.params.program_file)?;

        extended_domain.clone().for_each(|i| {
            let run_result = vm.run(i).unwrap();
            if self.select_witness(&run_result) {
                vset.push(i);
            }
        });

        Ok(Proof {
            vset,
            extended_domain: Some(extended_domain),
            params: self.params,
        })
    }

    /// Picking the witness based on the program result
    fn select_witness(&self, run_result: &RunResult) -> bool {
        if run_result.output != self.params.expected_output {
            return false;
        }

        if !self.params.input_domain.contains(&run_result.input) {
            return false;
        }

        validate_hash(&run_result.hash, self.params.kappa as usize)
    }
}
