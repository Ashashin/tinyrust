use color_eyre::{eyre::eyre, Report};
use sha1::{Digest, Sha1};
use structopt::StructOpt;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

use std::path::PathBuf;

mod parser;
mod vm;

use parser::Parser;
use vm::TinyVM;

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

/// Setup tracings and error reporting
fn setup() {
    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
/// Program entry point
fn main() -> Result<(), Report> {
    // General setup
    setup();

    // Process command-line arguments
    let opt = Opt::from_args();

    // Create VM
    let tinyvm = Parser::load_program(&opt.program_file)?;

    // Instantiate sha1 and add program and input tape to the hasher
    let mut hasher = Sha1::new();

    hasher.update(std::fs::read(opt.program_file)?);

    // Input handling
    let input = match opt.tape_file {
        Some(filename) => {
            let tape = Parser::load_tape_file(&filename)?;
            hasher.update(std::fs::read(filename)?);
            tape
        }
        _ => vec![19],
    };

    // Callback to update the hash
    let update_hash = |s: &[u8]| hasher.update(s);

    // Run program
    let output = run_vm(tinyvm, input, Some(update_hash))?;

    // Finalize hashing and write to file
    let hash = hasher.finalize();

    info!("output: {:?}, hash: {:?}", output, hash);

    Ok(())
}

fn run_vm<F>(mut tinyvm: TinyVM, input: Vec<usize>, callback: Option<F>) -> Result<usize, Report>
where
    F: FnMut(&[u8]),
{
    tinyvm.load_tape(input);

    info!("✨ All good to go! ✨");
    match tinyvm.run(callback)? {
        0 => {
            info!("✨ TinyVM terminated without error ✨");
            tinyvm.display_state();

            match tinyvm.output() {
                Some(&value) => Ok(value),
                _ => Err(eyre!("No output!")),
            }
        }
        x => Err(eyre!("🔥 Program terminated with error code {} 🔥", x)),
    }
}
