use color_eyre::Report;
use std::{fmt::Debug, ops::Range, path::Path};
use tinyvm::{parser::Parser, run_vm};

use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

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
    OverTesting,
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
    pub params: ProverParams,
}

impl Prover {
    pub fn new(params: ProverParams) -> Self {
        Self { params }
    }

    pub fn obtain_proof(&self) -> Result<Proof, Report> {
        match self.params.strategy {
            ProofStrategy::BestEffort | ProofStrategy::FixedEffort => {
                Ok(self.obtain_proof_best_effort()?)
            }
            _ => unimplemented!("Strategy unsupported: {:?}", self.params.strategy),
        }
    }

    fn obtain_proof_best_effort(&self) -> Result<Proof, Report> {
        let mut vset = vec![];
        let mut trials = 0;
        self.params.input_domain.clone().for_each(|i| {
            trials += 1;
            let run_result = run_instrumented_vm(self.params.program_file.clone(), i).unwrap();
            if self.select_witness(run_result) {
                vset.push(i);
            }
        });
        Ok(Proof {
            vset,
            params: self.params.clone(),
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

pub fn run_instrumented_vm<P>(filename: P, input: usize) -> Result<RunResult, Report>
where
    P: AsRef<Path> + Debug,
{
    use sha1::{Digest, Sha1};

    let vm = Parser::load_program(&filename)?;

    let mut hasher = Sha1::new();
    hasher.update(&std::fs::read(filename)?);
    let update_hash = |s: &[u8]| hasher.update(s);

    let output = run_vm(vm, vec![input], update_hash)?;

    let hash = hasher.finalize();
    let hash = hash.as_slice().to_vec();

    Ok(RunResult {
        input,
        output,
        hash,
    })
}
