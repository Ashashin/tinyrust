use color_eyre::{Help, Report};
use eyre::WrapErr;
use tracing::{info, instrument};
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

mod parser;
mod vm;

use parser::Parser;

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
    setup();

    let mut tinyvm = Parser::load_program("assets/test.tr")?;

    tinyvm.load_tape(vec![1, 2, 3]);
    tinyvm.run();

    Ok(())
}
