use color_eyre::Report;
use std::{fmt::Debug, ops::Range, path::Path, time::Instant};
use tinyvm::{parser::Parser, run_vm, TinyVM};

use crate::stats::compute_delta_u;
use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Debug)]
pub struct RunResult {
    pub hash: Vec<u8>,
    pub input: usize,
    pub output: usize,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ProofStrategy {
    FixedEffort,
    BestEffort,
    BestEffortAdaptive,
    OverTesting(f64),
}

pub struct Prover {
    params: ProverParams,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProverParams {
    pub program_file: String,
    pub input_domain: Range<usize>,
    pub expected_output: usize,
    pub kappa: u64,
    pub v: usize,
    pub strategy: ProofStrategy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proof {
    pub vset: Vec<usize>,
    pub extended_domain: Option<Range<usize>>,
    pub params: ProverParams,
}

impl Prover {
    pub fn new(params: ProverParams) -> Self {
        Self { params }
    }

    pub fn obtain_proof(self) -> Result<Proof, Report> {
        let start = Instant::now();
        let result = match self.params.strategy {
            ProofStrategy::BestEffort => self.obtain_proof_best_effort(),
            ProofStrategy::FixedEffort => self.obtain_proof_fixed_effort(),
            ProofStrategy::OverTesting(eta0) => self.obtain_proof_overtesting(eta0),
            ProofStrategy::BestEffortAdaptive => self.obtain_proof_bea(),
        };

        let duration = start.elapsed();

        println!("Prover time: {:?}", duration);

        result
    }

    fn obtain_proof_bea(self) -> Result<Proof, Report> {
        unimplemented!("Best effort adaptive strategy")
    }

    fn obtain_proof_fixed_effort(self) -> Result<Proof, Report> {
        self.obtain_proof_best_effort()
    }

    fn obtain_proof_best_effort(self) -> Result<Proof, Report> {
        let mut vset = vec![];
        let mut trials = 0;
        let domain = self.params.input_domain.clone();

        let mut vm = InstrumentedVM::new(&self.params.program_file)?;

        domain.for_each(|i| {
            trials += 1;
            let run_result = vm.run(i).unwrap();
            if self.select_witness(run_result) {
                vset.push(i);
            }
        });
        Ok(Proof {
            vset,
            extended_domain: None,
            params: self.params,
        })
    }

    fn obtain_proof_overtesting(self, eta0: f64) -> Result<Proof, Report> {
        let start = self.params.input_domain.start;
        let end = self.params.input_domain.end;

        let delta = compute_delta_u(eta0, self.params.kappa, end - start, self.params.v);
        let extended_domain = start..(end + delta);

        let mut vset = vec![];
        let mut trials = 0;

        let mut vm = InstrumentedVM::new(&self.params.program_file)?;

        extended_domain.clone().for_each(|i| {
            trials += 1;
            let run_result = vm.run(i).unwrap();
            if self.select_witness(run_result) {
                vset.push(i);
            }
        });
        Ok(Proof {
            vset,
            extended_domain: Some(extended_domain),
            params: self.params,
        })
    }

    fn select_witness(&self, run_result: RunResult) -> bool {
        if run_result.output != self.params.expected_output {
            return false;
        }

        if !self.params.input_domain.contains(&run_result.input) {
            return false;
        }

        validate_hash(run_result.hash, self.params.kappa as usize)
    }
}

pub fn validate_hash(hash: Vec<u8>, kappa: usize) -> bool {
    for hash_val in hash.view_bits::<Lsb0>().iter().take(kappa) {
        if !hash_val {
            return false;
        }
    }

    true
}

pub struct InstrumentedVM {
    vm: TinyVM,
    program: String,
}

impl InstrumentedVM {
    pub fn new<P>(filename: P) -> Result<Self, Report>
    where
        P: AsRef<Path> + Debug,
    {
        let vm = Parser::load_program(&filename)?;
        let program = serde_json::to_string(&vm.instructions())?;

        Ok(Self { vm, program })
    }

    pub fn run(&mut self, input: usize) -> Result<RunResult, Report> {
        let mut hasher = Sha1::new();
        hasher.update(&self.program);
        hasher.update(&input.to_be_bytes());
        let update_hash = |s: &[u8]| hasher.update(s);
        let output = run_vm(&mut self.vm, vec![input], update_hash)?;
        let hash = hasher.finalize();
        let hash = hash.as_slice().to_vec();

        Ok(RunResult {
            input,
            output,
            hash,
        })
    }
}
