# Oracle Omen

A production-grade Rust framework for deterministic, auditable, replayable autonomous agents with governed self-evolution.

## Vision

Oracle Omen is infrastructure for autonomous systems where nondeterminism is unacceptable:

> **Logs are truth. Replay is judge. Types are law.**

## Features

- **Deterministic by default**: Same inputs always produce same outputs
- **Full audit trail**: Every decision and action is logged
- **Replay capability**: Reproduce any execution exactly
- **Divergence detection**: Identify exactly where runs differ
- **Capability safety**: All tool access is gated by capabilities
- **Self-evolution**: Agents can propose and test their own improvements
- **CRDT memory**: Conflict-free replicated data structures
- **Provenance tracking**: Know why any fact exists in memory

## Quick Start

```toml
# Cargo.toml
[dependencies]
oracle_omen = "0.1"
```

```rust
use oracle_omen_core::{Agent, EventLog, CapabilitySet};
use oracle_omen_runtime::DagExecutor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent with capabilities
    let capabilities = CapabilitySet::new([
        Capability::new("fs:read:*"),
        Capability::new("network:http:get"),
    ]);

    // Run agent
    let run_id = run_agent(config, capabilities).await?;

    // Replay for verification
    let log = EventLog::load(run_id)?;
    let mut engine = ReplayEngine::new(log);
    let final_state = engine.replay_all()?;

    Ok(())
}
```

## Project Structure

```
oracle_omen/
├── crates/
│   ├── oracle_omen_core/      # Core types and abstractions
│   ├── oracle_omen_plan/      # Planning DSL and DAG compilation
│   ├── oracle_omen_runtime/   # Execution runtime
│   ├── oracle_omen_memory/    # CRDT memory store
│   └── oracle_omen_cli/       # Command-line interface
└── docs/
    ├── ARCHITECTURE.md        # System architecture
    ├── EVENT_LOG.md           # Event log specification
    ├── REPLAY.md              # Replay and divergence
    ├── CAPABILITIES.md        # Capability system
    ├── TOOLS.md               # Tool system
    ├── PLANNING.md            # Planning and DAG
    ├── MEMORY.md              # Memory CRDT
    ├── PATCHES.md             # Self-evolution
    └── CLI.md                 # CLI reference
```

## Milestones

| Milestone | Status | Description |
|-----------|--------|-------------|
| M0 | ✅ | Workspace skeleton |
| M1 | ✅ | Event log and stable hashing |
| M2 | ✅ | Agent core and state machine |
| M3 | ✅ | Tool runtime and capabilities |
| M4 | ✅ | Planner and DAG compiler |
| M5 | ✅ | Replay and diff engine |
| M6 | ✅ | Memory CRDT and provenance |
| M7 | 🔄 | Self-evolution patch system |
| M8 | ⏳ | WASM tool sandbox |

## Documentation

See the [docs](docs) folder for detailed documentation on each component.

## License

MIT

## Contributing

Contributions are welcome! Please read:
1. ARCHITECTURE.md for system design
2. Absolute rules in the codebase
3. Testing contract for requirements
