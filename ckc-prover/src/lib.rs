use color_eyre::Report;
use sha1::{Digest, Sha1};
use std::{fmt::Debug, path::Path};
use tinyvm::{parser::Parser, run_vm};

#[derive(Debug)]
pub struct RunResult {
    hash: Vec<u8>,
    output: usize,
}

pub struct Prover {
    params: ProverParams,
}
pub struct ProverParams {}

pub struct Proof {}

impl Prover {
    pub fn new(params: ProverParams) -> Self {
        todo!()
    }

    pub fn obtain_proof(&mut self) -> Proof {
        todo!()
    }

    fn select_witness(run_result: RunResult) -> bool {
        todo!()
    }
}

fn run_instrumented_vm<P>(filename: P, input: usize) -> Result<RunResult, Report>
where
    P: AsRef<Path> + Debug,
{
    let vm = Parser::load_program(&filename)?;

    let mut hasher = Sha1::new();
    hasher.update(&std::fs::read(filename)?);
    let update_hash = |s: &[u8]| hasher.update(s);

    let output = run_vm(vm, vec![input], update_hash)?;

    let hash = hasher.finalize();
    let hash = hash.as_slice().to_vec();

    Ok(RunResult { output, hash })
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

    #[test]
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
}
