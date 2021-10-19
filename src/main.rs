use color_eyre::{eyre::eyre, Help, Report};
use std::path::PathBuf;
use structopt::StructOpt;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

mod parser;
mod vm;

use parser::Parser;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Program file
    #[structopt(parse(from_os_str))]
    program_file: PathBuf,

    /// Tape file
    #[structopt(short, parse(from_os_str))]
    tape_file: Option<PathBuf>,

    /// Output file
    #[structopt(short, default_value = "cert.out")]
    outfile: String,
}

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
fn main() -> Result<(), Report> {
    // General setup
    setup();
    let opt = Opt::from_args();

    // Load program
    let mut tinyvm = Parser::load_program(opt.program_file)?;

    // Load tape
    if let Some(filename) = opt.tape_file {
        tinyvm.load_tape(Parser::load_tape_file(filename)?);
    }

    // Run program
    info!("All good to go!");

    match tinyvm.run()? {
        0 => {
            info!("TinyVM terminated without error");
            tinyvm.display_state();
            Ok(())
        }
        x => Err(eyre!("Program terminated with error code {}", x)),
    }
}
