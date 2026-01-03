# Memory System

## Overview

The memory system provides:
- CRDT document store
- Causal event linkage
- Temporal queries
- Deterministic retrieval

## Document Structure

```rust
pub struct Document {
    pub key: String,
    pub value: DocumentValue,
    pub version: BTreeMap<String, u64>,
    pub causal_event: u64,
    pub hash: Hash,
}
```

## Document Values

| Type | Description |
|------|-------------|
| `String` | Text data |
| `Bytes` | Binary data |
| `Integer` | Signed 64-bit integer |
| `Float` | Floating point (use carefully) |
| `Bool` | Boolean |
| `Map` | Nested map |
| `Vec` | List of values |
| `Null` | Empty value |
| `Ref` | Hash reference |

## CRDT Semantics

Documents use LWW (Last-Writer-Wins) semantics:

```rust
// Later causal event wins
if doc2.causal_event > doc1.causal_event {
    merged = doc2;
} else {
    merged = doc1;
}
```

## Provenance Tracking

Every memory write records its causal event:

```rust
pub struct ProvenanceRecord {
    pub causal_event: u64,
    pub operation: Operation,
    pub key: String,
    pub value_hash: Option<String>,
    pub timestamp: u64,
}
```

## Temporal Queries

Query state at a specific event:

```rust
let state_at_n = store.state_at_event(event_id: 100);
```

## Deterministic Retrieval

Keys are always returned in sorted order:

```rust
let keys = store.keys_sorted();  // BTreeMap ensures order
```

## Query API

```rust
// Build query
let results = QueryBuilder::new(&store)
    .filter(Filter::KeyPrefix("user:".to_string()))
    .filter(Filter::TypeEquals("string".to_string()))
    .order_by(QueryOrder::Key)
    .limit(10)
    .execute()?;
```

## Invariants

1. **Causal linkage**: Every write links to an event
2. **Deterministic order**: BTreeMap ensures stable iteration
3. **Merge safety**: CRDT merge is commutative and associative
4. **Provenance**: Can trace why any value exists

## Failure Modes

### Merge Conflict
- **Detection**: Different causal events claim same key
- **Resolution**: LWW semantics (later wins)
- **Prevention**: Single writer per key when possible

### Corruption
- **Detection**: Hash mismatch on document
- **Recovery**: Reconstruct from event log
- **Prevention**: Hash verification on write
