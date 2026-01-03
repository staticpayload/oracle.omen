# Replay and Divergence Detection

## Overview

The replay engine reconstructs agent execution from event logs, enabling:
- Failure reproduction
- Debugging
- Auditing
- Divergence detection

## Replay Process

```rust
let mut engine = ReplayEngine::new(event_log);
let final_state = engine.replay_all()?;
```

### Step-by-Step Replay

```rust
while let Some(event) = engine.step() {
    // Inspect event
    println!("{:?}", event);
}
```

### Partial Replay

```rust
// Load snapshot at position 1000
let snapshot = load_snapshot(1000)?;
let mut engine = ReplayEngine::with_state(log, snapshot.state);
engine.replay_from(1000)?;
```

## Divergence Detection

When comparing two runs:

```rust
let engine1 = ReplayEngine::new(log1);
let engine2 = ReplayEngine::new(log2);

let divergences = engine1.detect_divergence(&engine2);
for point in divergences {
    println!("Divergence at {}: {}", point.position, point.diff);
}
```

## Divergence Point

```rust
pub struct DivergencePoint {
    pub position: u64,      // Event number where divergence occurred
    pub event_id: EventId,
    pub expected: Hash,     // Hash from first run
    pub actual: Hash,       // Hash from second run
    pub diff: String,       // Human-readable diff
}
```

## Verification

After replay, verify integrity:

```rust
let report = engine.verify()?;
println!("{}", report);
```

Output:
```
Verification: 1523/1523 events verified, 0 failures, 0 mismatches - VALID
```

## Snapshot Strategy

Snapshots accelerate replay:

1. **Periodic snapshots**: Every N events
2. **Key positions**: Before/after critical operations
3. **State-based**: When state reaches certain size

```rust
let snapshot = Snapshot::new(id, run_id, position, state.clone());
snapshot_manager.add(snapshot);
```

## Invariants

1. **Deterministic replay**: Same log produces same state
2. **Hash verification**: Every event must verify
3. **Divergence detection**: Single bit change detected
4. **Snapshot integrity**: State hash must match

## Failure Modes

### Replay Failure
- **Cause**: Corrupted event or missing data
- **Detection**: Hash mismatch during replay
- **Recovery**: Use earlier snapshot

### Divergence
- **Cause**: Non-deterministic tool or external factor
- **Detection**: Different hash at same position
- **Recovery**: Identify tool, fix or mark as non-deterministic
