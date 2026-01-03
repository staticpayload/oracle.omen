# Oracle Omen Architecture

## Overview

Oracle Omen is a production-grade Rust framework for deterministic, auditable, replayable autonomous agents with governed self-evolution.

## Core Principles

1. **Determinism is mandatory** - Same inputs always produce same outputs
2. **Side effects must be explicit** - All IO goes through capability-gated tools
3. **All tool calls are logged** - Every action has an audit trail
4. **All state transitions are logged** - Full history reconstruction is possible
5. **No silent mutation** - All changes are recorded
6. **No global state** - State is explicit and passed around
7. **No implicit permissions** - Capabilities are checked before every tool call
8. **No opaque magic** - Everything is transparent and inspectable
9. **No panics in runtime paths** - Errors are data, not crashes
10. **No unordered iteration where it matters** - Use BTreeMap for deterministic ordering

## Crate Structure

```
oracle_omen/
├── oracle_omen_core      # Pure logic, no IO
├── oracle_omen_plan      # Planning DSL and DAG compilation
├── oracle_omen_runtime   # IO, tools, scheduler, capability enforcement
├── oracle_omen_memory    # CRDT, provenance, retrieval ordering
└── oracle_omen_cli       # Interface and presentation
```

## Core Abstractions

### Agent

An agent is a pure state machine:

```
Input: prior state, observation, tool responses, injected deterministic context
Output: next state, planned actions, patch proposals
```

Agents cannot directly perform IO. All IO goes through tools.

### Event Log

Append-only log of all events:

```
- event_id: stable identifier (run_id:sequence)
- parent_event_id: causal linkage
- timestamp: injected logical time
- kind: event type
- payload: event data
- payload_hash: for verification
- state_hash_before/after: for state tracking
```

### State

Agent state is a typed container:

```
- version: increments on each transition
- data: BTreeMap<String, StateData> for deterministic ordering
- state_hash: for verification
```

### Tool

Tools declare:

```
- name and version
- input schema and output schema
- required capabilities
- side effect declaration (Pure/Impure)
- determinism declaration
- timeout and resource bounds
```

### Capability

Capabilities grant permission:

```
- Format: domain:action:scope
- Example: fs:read:*, network:http:get
- Immutable during execution run
- Checked before tool execution
```

## Invariants

1. **Event ordering**: Events are ordered by sequence within a run
2. **Hash stability**: Same data always produces same hash
3. **Serialization stability**: BTreeMap ensures stable key ordering
4. **Time monotonicity**: Logical time only increases
5. **State transition validity**: Transitions are logged and verifiable
6. **Capability enforcement**: Tools cannot run without capabilities

## Failure Modes

### Event Log Corruption
- Detection: Hash mismatch on replay
- Recovery: Restore from last valid snapshot
- Prevention: Atomic writes, hash verification

### State Divergence
- Detection: Different hash on replay
- Recovery: Diff tool identifies divergence point
- Prevention: Deterministic tools, seeded randomness

### Capability Escalation
- Detection: Audit log shows tool without capability
- Recovery: Event marked as denied, execution stops
- Prevention: Strict capability checking at runtime

### Non-determinism
- Detection: Different outputs on replay
- Recovery: Tool marked as non-deterministic
- Prevention: Tool declarations, testing

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         Agent                                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │    State    │ -> │  Planning   │ -> │     Decision        │ │
│  └─────────────┘    └─────────────┘    └─────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Runtime                                    │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │   Capabi-   │ -> │   Tool      │ -> │      Event          │ │
│  │  lity Check │    │  Execution  │    │      Logging        │ │
│  └─────────────┘    └─────────────┘    └─────────────────────┘ │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Memory Store                               │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │    CRDT     │ <- │ Provenance  │ <- │    Causal Links     │ │
│  │  Documents  │    │   Tracking  │    │                     │ │
│  └─────────────┘    └─────────────┘    └─────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Execution Model

1. **Initialize**: Create initial state, event log, capability set
2. **Observe**: Read input from environment (logged as event)
3. **Decide**: Agent produces decision based on state + observation
4. **Execute**: Run tools with capability checks (logged as events)
5. **Update**: Apply tool responses to state (logged as event)
6. **Repeat**: Go to step 2 until termination
7. **Snapshot**: Save state and event position for efficient replay

## Replay Process

1. Load event log
2. Load snapshot (if available) to skip early events
3. Replay events from snapshot position
4. Verify each event hash
5. Reconstruct state
6. Compare final state hash with original
