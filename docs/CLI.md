# Oracle Omen CLI

## Overview

The `oracle-omen` CLI provides commands for running agents, replaying execution, and inspecting state.

## Installation

```bash
cargo install oracle-omen
```

## Commands

### Run

Execute an agent:

```bash
oracle-omen run config.toml
```

Options:
- `-d, --data-dir <DIR>`: Data directory (default: `.oracle-omen`)
- `-v, --verbose`: Verbose output

### Replay

Replay an execution:

```bash
oracle-omen replay <run_id>
```

Shows:
- Events replayed
- State reconstruction
- Verification results

### Trace

Show execution trace:

```bash
oracle-omen trace <run_id>
```

Shows:
- All events with IDs
- Timestamps
- Hashes
- Payload summaries

### Diff

Compare two runs:

```bash
oracle-omen diff <run_a> <run_b>
```

Shows:
- First divergence point
- Detailed diff
- State comparison

### Inspect

Inspect a run:

```bash
oracle-omen inspect <run_id>
```

Shows:
- Run metadata
- Event summary
- Final state
- Capability usage

### Capabilities

List capabilities:

```bash
oracle-omen capabilities <run_id>
```

Shows:
- Granted capabilities
- Usage statistics
- Denials

## Data Directory

The data directory contains:

```
.oracle-omen/
├── runs/
│   ├── <run_id>/
│   │   ├── events.jsonl    # Event log
│   │   ├── snapshot.bin    # State snapshot
│   │   └── meta.json       # Run metadata
```

## Output Formats

### Table Output

```
+----------+------------+------------+
| Event ID | Kind       | Hash       |
+----------+------------+------------+
| E(1:0)   | AgentInit  | a1b2c3...  |
| E(1:1)   | Observation| d4e5f6...  |
+----------+------------+------------+
```

### JSON Output

```bash
oracle-omen --output json inspect <run_id>
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error |
| 2 | Invalid arguments |
| 3 | Run not found |
| 4 | Verification failed |
