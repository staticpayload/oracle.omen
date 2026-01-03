//! oracle-omen CLI
//!
//! Deterministic, auditable autonomous agents.

use clap::Parser;

mod commands;
mod output;

use commands::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}
