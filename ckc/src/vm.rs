use bitvec::prelude::*;
use color_eyre::Report;
use sha1::{Digest, Sha1};

use std::{fmt::Debug, path::Path};

use tinyvm::{parser::Parser, TinyVM};

#[derive(Debug)]
pub struct RunResult {
    pub hash: Vec<u8>,
    pub input: usize,
    pub output: usize,
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
        let update_hash = |s: &[u8]| hasher.update(s);
        let output = self.vm.run_vm_with_callback(vec![input], update_hash)?;
        let hash = hasher.finalize();
        let hash = hash.as_slice().to_vec();
        self.vm.reset_state();

        Ok(RunResult {
            input,
            output,
            hash,
        })
    }
}

pub fn validate_hash(hash: Vec<u8>, kappa: usize) -> bool {
    for hash_val in hash.view_bits::<Msb0>().iter().take(160 - kappa) {
        if *hash_val {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_fib_with_instrumentation() -> Result<(), Report> {
        let mut vm = InstrumentedVM::new(&String::from("../assets/fib.tr"))?;
        let result = vm.run(39)?;
        println!("Result = {:?}", result);

        let expected_output = 63245986;

        assert_eq!(result.output, expected_output);

        Ok(())
    }

    #[test]
    fn run_collatz_with_instrumentation() -> Result<(), Report> {
        let mut vm = InstrumentedVM::new(&String::from("../assets/collatz_v0.tr"))?;

        let result = vm.run(39)?;
        println!("Result = {:?}", result);

        let expected_output = 0;

        assert_eq!(result.output, expected_output);

        Ok(())
    }
}
