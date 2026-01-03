//! oracle-omen CLI
//!
//! Deterministic, auditable autonomous agents.

use clap::Parser;
use std::process::ExitCode;

mod commands;
mod output;

use commands::{Cli, CliError};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
