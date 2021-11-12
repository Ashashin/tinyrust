use color_eyre::Report;
use structopt::StructOpt;
use tracing::info;

use std::path::PathBuf;

pub mod parser;
pub mod vm;

use parser::Parser;
pub use vm::TinyVM;

/// Command line options
#[derive(Debug, StructOpt)]
struct Opt {
    /// Program file
    #[structopt(parse(from_os_str))]
    program_file: PathBuf,

    /// Tape file
    #[structopt(short, parse(from_os_str))]
    tape_file: Option<PathBuf>,
}

/// Program entry point
pub fn from_cli() -> Result<(), Report> {
    // Process command-line arguments
    let opt = Opt::from_args();

    // Create VM
    let mut tinyvm = Parser::load_program(&opt.program_file)?;

    // Input handling
    let input = match opt.tape_file {
        Some(filename) => Parser::load_tape_file(&filename)?,
        _ => vec![27],
    };

    // Run program
    let output = tinyvm.run_vm((input, vec![]))?;

    info!("output: {:?}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use sha1::{Digest, Sha1};

    use crate::Parser;
    use color_eyre::Report;

    #[test]
    fn run_fibo() -> Result<(), Report> {
        let mut vm = Parser::load_program(&String::from("../assets/fib.tr"))?;
        let result = vm.run_vm((vec![39], vec![]))?;
        println!("Result = {}", result);

        assert_eq!(result, 63245986);
        Ok(())
    }

    #[test]
    fn run_fib_with_callback() -> Result<(), Report> {
        let mut hasher = Sha1::new();
        let update_hash = |s: &[u8]| hasher.update(s);

        let mut vm = Parser::load_program(&String::from("../assets/fib.tr"))?;
        let result = vm.run_vm_with_callback((vec![39], vec![]), update_hash)?;

        let hash = hasher.finalize();
        let expected_output = 63245986;

        println!("Result = {:?}", result);
        println!("Hash = {:?}", hash);

        assert_eq!(result, expected_output);

        Ok(())
    }

    #[test]
    fn run_collatz_with_callback() -> Result<(), Report> {
        let mut hasher = Sha1::new();
        let update_hash = |s: &[u8]| hasher.update(s);

        let mut vm = Parser::load_program(&String::from("../assets/collatz_v0.tr"))?;
        let result = vm.run_vm_with_callback((vec![39], vec![]), update_hash)?;

        let hash = hasher.finalize();
        let expected_output = 0;

        println!("Result = {:?}", result);
        println!("Hash = {:?}", hash);

        assert_eq!(result, expected_output);

        Ok(())
    }
}
