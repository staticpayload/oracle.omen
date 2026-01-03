// oracle_omen_cli: Interface and presentation only
//
// CLI for oracle-omen framework

#![warn(missing_docs)]
#![warn(clippy::all)]

mod commands;
mod output;

pub use commands::*;
pub use output::*;
