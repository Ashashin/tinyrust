use bitvec::prelude::*;
use color_eyre::Report;
use sha1::{Digest, Sha1};

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::stats::compute_q;
use tinyvm::{parser::Parser, TinyVM};

/// Strucr reprensenting the result of the instrumented VM run
#[derive(Debug, Clone)]
pub struct RunResult {
    /// Hash of the program run
    pub hash: Vec<u8>,
    /// Input value
    pub input: usize,
    /// Program output
    pub output: usize,
}

/// VM used in CKC to hash the different states
pub struct InstrumentedVM {
    /// The VM instance
    vm: TinyVM,
    /// The executed program
    program: String,
}

impl InstrumentedVM {
    /// Create a new VM for a given program
    pub fn new<P>(filename: P) -> Result<Self, Report>
    where
        P: AsRef<Path> + Debug,
    {
        let vm = Parser::load_program(&filename)?;
        let program = serde_json::to_string(&vm.instructions())?;

        Ok(Self { vm, program })
    }

    /// Run the VM with the given input
    pub fn run(&mut self, input: usize) -> Result<RunResult, Report> {
        let mut hasher = Sha1::new();
        hasher.update(&self.program);
        let update_hash = |s: &[u8]| hasher.update(s);
        let output = self
            .vm
            .run_vm_with_callback((vec![input], vec![]), update_hash)?;
        let hash = hasher.finalize();
        let hash = hash.as_slice().to_vec();
        self.vm.reset_state();

        Ok(RunResult {
            hash,
            input,
            output,
        })
    }
}

/// Validate the output hash
pub fn validate_hash(hash: &[u8], kappa: usize) -> bool {
    for hash_val in hash.view_bits::<Msb0>().iter().take(160 - kappa) {
        if *hash_val {
            return false;
        }
    }

    true
}

pub fn get_data(
    program: PathBuf,
    u: usize,
    u_max: usize,
) -> Result<Vec<(usize, Vec<f64>)>, Report> {
    let kappa_min = 144;
    let kappa_max = 159;
    let kappa_num = 5;
    let get_kappa = |i: usize| (kappa_max - kappa_min) * i / (kappa_num - 1) + kappa_min;

    let mut data: Vec<(usize, Vec<f64>)> = (0..kappa_num)
        .map(|i| (get_kappa(i), vec![0.0; u_max]))
        .collect();

    let start = Instant::now();
    let mut vm = InstrumentedVM::new(program)?;

    // Accumulator for the valid number of hashes
    let mut acc: Vec<usize> = vec![0; kappa_num];

    // Create data points form vm run
    (0..u_max).for_each(|i| {
        let h = vm.run(i).unwrap().hash;

        // Apply each hash to a kappa
        data.iter_mut()
            .enumerate()
            .for_each(|(idx, (kappa, values))| {
                if validate_hash(&h, *kappa) {
                    acc[idx] += 1;
                }
                values[i] = compute_q(*kappa as u64, u, acc[idx]);
            })
    });

    println!("Got traces in: {:?}", start.elapsed());

    Ok(data)
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
