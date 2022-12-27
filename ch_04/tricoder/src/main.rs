use anyhow::Result;
use args::Args;

mod args;
mod cli;
mod common_ports;
mod dns;
mod error;
mod modules;
mod ports;
use clap::Parser;
pub use error::Error;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args {
        Args::Modules => cli::modules(),
        Args::Scan { target } => cli::scan(&target)?,
    }

    Ok(())
}
