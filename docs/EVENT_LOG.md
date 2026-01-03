# Event Log Specification

## Overview

The event log is the source of truth for all agent execution. It enables deterministic replay, auditability, and divergence detection.

## Event Structure

```rust
pub struct Event {
    pub id: EventId,           // Unique: run_id:sequence
    pub parent_id: Option<EventId>,
    pub kind: EventKind,
    pub timestamp: LogicalTime,
    pub payload: EventPayload,
    pub payload_hash: Hash,
    pub state_hash_before: Option<Hash>,
    pub state_hash_after: Option<Hash>,
}
```

## Event ID

Events are identified by `run_id:sequence`:

```
run_id:    Unique identifier for an execution run
sequence:  Monotonically increasing counter starting at 0
```

Example: `E(42:5)` = 5th event of run 42

## Event Kinds

| Kind | Description | Payload Type |
|------|-------------|--------------|
| `AgentInit` | Agent initialized | `AgentInitPayload` |
| `StateTransition` | State changed | `StateTransitionPayload` |
| `ToolRequest` | Tool execution requested | `ToolRequestPayload` |
| `ToolResponse` | Tool response received | `ToolResponsePayload` |
| `CapabilityDenied` | Capability check failed | `CapabilityDeniedPayload` |
| `Observation` | Environment observation | `ObservationPayload` |
| `Decision` | Agent decision | `DecisionPayload` |
| `MemoryWrite` | Memory written | `MemoryPayload` |
| `MemoryRead` | Memory read | `MemoryPayload` |
| `PatchProposal` | Self-patch proposed | `PatchPayload` |
| `PatchApplied` | Patch applied | `PatchPayload` |
| `PatchRejected` | Patch rejected | `PatchRejectedPayload` |
| `Error` | Error occurred | `ErrorPayload` |
| `Snapshot` | Snapshot created | `SnapshotPayload` |

## Payload Definitions

### AgentInitPayload

```rust
pub struct AgentInitPayload {
    pub agent_type: String,
    pub agent_version: String,
    pub config: BTreeMap<String, String>,
}
```

### ToolRequestPayload

```rust
pub struct ToolRequestPayload {
    pub tool_name: String,
    pub tool_version: String,
    pub request_hash: Hash,
    pub capabilities: Vec<Capability>,
    pub input: String,  // JSON string for stability
}
```

### ToolResponsePayload

```rust
pub struct ToolResponsePayload {
    pub tool_name: String,
    pub request_hash: Hash,
    pub response_hash: Hash,
    pub output: String,     // JSON string
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}
```

## Invariants

1. **Sequence continuity**: Events are numbered 0, 1, 2, ... without gaps
2. **Parent exists**: `parent_id` must reference a valid earlier event
3. **Hash validity**: `payload_hash` must match `hash(payload)`
4. **Time monotonicity**: Timestamps never decrease

## Failure Modes

### Corrupted Log
- **Detection**: Event sequence mismatch or hash failure
- **Recovery**: Restore from last valid snapshot
- **Prevention**: Atomic appends, immediate verification

### Parent Not Found
- **Detection**: Parent event ID not in index
- **Recovery**: Reject event, log error
- **Prevention**: Verify parent exists before append

### Hash Mismatch
- **Detection**: `payload_hash != hash(payload)`
- **Recovery**: Reject event, investigate tampering
- **Prevention**: Compute hash during creation
