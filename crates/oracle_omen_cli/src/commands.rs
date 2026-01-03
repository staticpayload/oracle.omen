//! CLI commands for oracle-omen.

use std::path::PathBuf;
use crate::output::Output;

/// CLI commands
#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Run an agent
    Run {
        /// Configuration file
        config: PathBuf,
    },

    /// Replay a run
    Replay {
        /// Run ID to replay
        run_id: String,
    },

    /// Trace a run
    Trace {
        /// Run ID to trace
        run_id: String,
    },

    /// Diff two runs
    Diff {
        /// First run ID
        run_a: String,
        /// Second run ID
        run_b: String,
    },

    /// Inspect a run
    Inspect {
        /// Run ID to inspect
        run_id: String,
    },

    /// List capabilities
    Capabilities {
        /// Run ID
        run_id: String,
    },
}

/// Main CLI struct
#[derive(Debug, clap::Parser)]
#[command(name = "oracle-omen")]
#[command(about = "Deterministic, auditable autonomous agents", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Data directory
    #[arg(short, long, default_value = ".oracle-omen")]
    pub data_dir: PathBuf,
}

impl Cli {
    /// Run the CLI
    pub fn run(&self) -> Result<(), CliError> {
        match &self.command {
            Command::Run { config } => commands::run(self, config),
            Command::Replay { run_id } => commands::replay(self, run_id),
            Command::Trace { run_id } => commands::trace(self, run_id),
            Command::Diff { run_a, run_b } => commands::diff(self, run_a, run_b),
            Command::Inspect { run_id } => commands::inspect(self, run_id),
            Command::Capabilities { run_id } => commands::capabilities(self, run_id),
        }
    }
}

/// CLI errors
#[derive(Clone, Debug)]
pub enum CliError {
    /// IO error
    Io(String),

    /// Config error
    Config(String),

    /// Runtime error
    Runtime(String),

    /// Not found
    NotFound(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Io(msg) => write!(f, "IO error: {}", msg),
            CliError::Config(msg) => write!(f, "Config error: {}", msg),
            CliError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            CliError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e.to_string())
    }
}

/// Command implementations
mod commands {
    use super::*;

    pub fn run(cli: &Cli, config: &PathBuf) -> Result<(), CliError> {
        let output = Output::new()
            .header("oracle-omen run")
            .kv("config", config.display())
            .kv("data_dir", cli.data_dir.display())
            .line("")
            .line("Loading agent configuration...")
            .line("Initializing event log...")
            .line("Starting agent execution...");

        if cli.verbose {
            output.print();
        }

        // TODO: Actual implementation
        // 1. Load config from file
        // 2. Initialize event log
        // 3. Create agent instance
        // 4. Run agent loop
        // 5. Log all events
        // 6. Save final state

        Ok(())
    }

    pub fn replay(cli: &Cli, run_id: &str) -> Result<(), CliError> {
        let output = Output::new()
            .header("oracle-omen replay")
            .kv("run_id", run_id)
            .line("")
            .line("Loading event log...")
            .line("Replaying events...")
            .line("Verifying state reconstruction...");

        if cli.verbose {
            output.print();
        }

        // TODO: Actual implementation
        // 1. Load event log for run_id
        // 2. Load snapshot if available
        // 3. Replay from snapshot or beginning
        // 4. Verify each event hash
        // 5. Compare final state hash

        Ok(())
    }

    pub fn trace(cli: &Cli, run_id: &str) -> Result<(), CliError> {
        let output = Output::new()
            .header("oracle-omen trace")
            .kv("run_id", run_id)
            .line("")
            .section("Events")
            .line("Loading event log...");

        if cli.verbose {
            output.print();
        }

        // TODO: Actual implementation
        // 1. Load event log
        // 2. Print each event with:
        //    - Event ID
        //    - Kind
        //    - Timestamp
        //    - Hashes
        //    - Payload summary

        Ok(())
    }

    pub fn diff(cli: &Cli, run_a: &str, run_b: &str) -> Result<(), CliError> {
        let output = Output::new()
            .header("oracle-omen diff")
            .kv("run_a", run_a)
            .kv("run_b", run_b)
            .line("")
            .section("Divergence Analysis")
            .line("Comparing event logs...");

        output.print();

        // TODO: Actual implementation
        // 1. Load both event logs
        // 2. Find first divergence point
        // 3. Show detailed diff
        // 4. Compare final states

        Ok(())
    }

    pub fn inspect(cli: &Cli, run_id: &str) -> Result<(), CliError> {
        let output = Output::new()
            .header("oracle-omen inspect")
            .kv("run_id", run_id)
            .line("")
            .section("Run Information")
            .line("Loading run data...")
            .line("")
            .section("Event Summary")
            .line("Loading events...")
            .line("")
            .section("State")
            .line("Loading final state...");

        if cli.verbose {
            output.print();
        }

        // TODO: Actual implementation
        // 1. Load run metadata
        // 2. Show event count and types
        // 3. Show final state
        // 4. Show capability usage

        Ok(())
    }

    pub fn capabilities(cli: &Cli, run_id: &str) -> Result<(), CliError> {
        let output = Output::new()
            .header("oracle-omen capabilities")
            .kv("run_id", run_id)
            .line("")
            .section("Granted Capabilities")
            .line("Loading capability set...")
            .line("")
            .section("Usage Summary")
            .line("Analyzing capability usage...");

        output.print();

        // TODO: Actual implementation
        // 1. Load capability set
        // 2. Show granted capabilities
        // 3. Show tool calls per capability
        // 4. Show denials if any

        Ok(())
    }
}
