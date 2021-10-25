use color_eyre::Report;
use std::{fmt::Debug, ops::Range, path::Path};
use tinyvm::{parser::Parser, run_vm};

use bitvec::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct RunResult {
    hash: Vec<u8>,
    input: usize,
    output: usize,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ProofStrategy {
    FixedEffort,
    BestEffort,
    BestEffortAdaptive,
    OverTesting,
    ReTestingSalt,
    ReTestingObfuscation,
}

pub struct Prover {
    params: ProverParams,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProverParams {
    program_file: String,
    input_domain: Range<usize>,
    expected_output: usize,
    kappa: u64,
    strategy: ProofStrategy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proof {
    vset: Vec<usize>,
    params: ProverParams,
}

impl Prover {
    pub fn new(params: ProverParams) -> Self {
        Self { params }
    }

    pub fn obtain_proof(&self) -> Result<Proof, Report> {
        match self.params.strategy {
            ProofStrategy::BestEffort => {
                let mut vset = vec![];
                let mut trials = 0;
                self.params.input_domain.clone().for_each(|i| {
                    trials += 1;
                    let run_result =
                        run_instrumented_vm(self.params.program_file.clone(), i).unwrap();
                    if self.select_witness(run_result) {
                        vset.push(i);
                    }
                });
                Ok(Proof {
                    vset,
                    params: self.params.clone(),
                })
            }
            _ => unimplemented!("Strategy unsupported: {:?}", self.params.strategy),
        }
    }

    fn select_witness(&self, run_result: RunResult) -> bool {
        if run_result.output != self.params.expected_output {
            return false;
        }

        if !self.params.input_domain.contains(&run_result.input) {
            return false;
        }

        for hash_val in run_result
            .hash
            .view_bits::<Lsb0>()
            .iter()
            .take(self.params.kappa as usize)
        {
            if !hash_val {
                return false;
            }
        }

        true
    }
}

fn run_instrumented_vm<P>(filename: P, input: usize) -> Result<RunResult, Report>
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn run_fib() -> Result<(), Report> {
        let update_hash = |_: &[u8]| {};

        let vm = Parser::load_program(&String::from("../assets/fib.tr"))?;
        let result = run_vm(vm, vec![39], update_hash)?;
        println!("Result = {}", result);

        assert_eq!(result, 63245986);
        Ok(())
    }

    //#[test]
    fn run_fib_with_instrumentation() -> Result<(), Report> {
        let result = run_instrumented_vm(&String::from("../assets/fib.tr"), 39)?;
        println!("Result = {:?}", result);

        let expected_output = 63245986;
        let expected_hash = vec![
            102, 171, 177, 23, 197, 105, 13, 18, //
            161, 113, 165, 119, 114, 1, 250, 51, //
            54, 239, 253, 9,
        ];

        assert_eq!(result.output, expected_output);
        assert_eq!(result.hash, expected_hash);

        Ok(())
    }

    #[test]
    fn run_collatz() -> Result<(), Report> {
        let update_hash = |_: &[u8]| {};

        let vm = Parser::load_program(&String::from("../assets/collatz_v0.tr"))?;
        let result = run_vm(vm, vec![8], update_hash)?;
        println!("Result = {}", result);

        assert_eq!(result, 0);
        Ok(())
    }

    #[test]
    fn run_collatz_with_instrumentation() -> Result<(), Report> {
        let result = run_instrumented_vm(&String::from("../assets/collatz_v0.tr"), 39)?;
        println!("Result = {:?}", result);

        let expected_output = 0;
        let expected_hash = vec![
            104, 215, 153, 215, 16, 249, 82, 66, 165, 150, 84, 196, 233, 89, 118, 123, 255, 20,
            107, 110,
        ];

        assert_eq!(result.output, expected_output);
        assert_eq!(result.hash, expected_hash);

        Ok(())
    }

    #[test]
    fn run_proof() -> Result<(), Report> {
        let prover = Prover::new(ProverParams {
            program_file: String::from("../assets/collatz_v0.tr"),
            input_domain: 1..1000,
            expected_output: 0,
            strategy: ProofStrategy::BestEffort,
            kappa: 14,
        });

        let proof = prover.obtain_proof()?;

        println!("Proof = {:?}", proof);

        Ok(())
    }
}
