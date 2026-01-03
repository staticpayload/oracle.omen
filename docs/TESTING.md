# Test Coverage

## Overview

Comprehensive testing is required. Every bug must gain a regression test.

## Test Organization

```
crates/
├── oracle_omen_core/
│   └── tests/
│       ├── event_log_tests.rs
│       ├── hash_tests.rs
│       ├── state_tests.rs
│       └── replay_tests.rs
├── oracle_omen_plan/
│   └── tests/
│       ├── dsl_tests.rs
│       ├── dag_tests.rs
│       └── compiler_tests.rs
├── oracle_omen_runtime/
│   └── tests/
│       ├── capability_tests.rs
│       ├── scheduler_tests.rs
│       └── tool_tests.rs
├── oracle_omen_memory/
│   └── tests/
│       ├── crdt_tests.rs
│       ├── provenance_tests.rs
│       └── query_tests.rs
├── oracle_omen_policy/
│   └── tests/
│       ├── policy_tests.rs
│       └── engine_tests.rs
├── oracle_omen_patches/
│   └── tests/
│       ├── gate_tests.rs
│       └── apply_tests.rs
└── oracle_omen_wasm/
    └── tests/
        ├── sandbox_tests.rs
        └── compile_tests.rs
```

## Required Test Categories

### 1. Unit Tests

- Every public function
- Every error path
- Edge cases (empty, single item, max items)

### 2. Integration Tests

- Event log append and replay
- Full agent execution cycle
- Tool execution with capabilities
- Memory operations with provenance

### 3. Property Tests

- Hash stability (always same output)
- Serialization round-trip
- BTreeMap ordering
- CRDT convergence

### 4. Fuzzing Targets

- Event log parser
- Hash computation (edge cases)
- WASM tool execution
- Policy condition evaluation

### 5. Regression Tests

One test per discovered bug:
```rust
#[test]
fn test_regression_issue_123() {
    // Test for bug fixed in issue #123
    let result = problematic_case();
    assert!(result.is_ok());
}
```

## Coverage Goals

| Component | Target Coverage |
|-----------|-----------------|
| Core | 95%+ |
| Plan | 90%+ |
| Runtime | 90%+ |
| Memory | 90%+ |
| Policy | 85%+ |
| Patches | 85%+ |
| WASM | 85%+ |

## Running Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

## Acceptance Tests

### M0: Workspace
```bash
cargo test
cargo clippy
cargo fmt --check
```

### M1: Event Log
```bash
cargo test --test event_log_stable_serialization
cargo test --test event_log_hash_verification
```

### M2: State Machine
```bash
cargo test --test state_transitions
cargo test --test state_reproducibility
```

### M3: Tools
```bash
cargo test --test capability_denial
cargo test --test tool_call_logging
```

### M4: Planning
```bash
cargo test --test dag_compile
cargo test --test topological_order
```

### M5: Replay
```bash
cargo test --test replay_identity
cargo test --test divergence_diff
```

### M6: Memory
```bash
cargo test --test causal_links
cargo test --test temporal_query
cargo test --test deterministic_retrieval
```

### M7: Patches
```bash
cargo test --test patch_signature
cargo test --test test_gate
cargo test --test patch_apply_rollback
```

### M8: WASM
```bash
cargo test --test wasm_fuel_limit
cargo test --test wasm_deterministic_result
```

## Property Based Tests

```rust
proptest! {
    #[test]
    fn prop_event_hash_stable(events in prop::collection::vec(any::<Event>(), 0..100)) {
        let log = EventLog::from_events(&events);
        let h1 = log.hash();
        let h2 = log.hash();
        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn prop_btree_ordering(items in prop::collection::btree_map(any::<String>(), any::<Value>())) {
        let keys1: Vec<_> = items.keys().collect();
        let keys2: Vec<_> = items.keys().collect();
        prop_assert_eq!(keys1, keys2);
    }
}
```

## Fuzzing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Add fuzz target
cargo fuzz add event_log_parser

# Run fuzzing
cargo fuzz run event_log_parser fuzz/
```

## Continuous Integration

CI must enforce:
- All tests pass
- Coverage threshold met
- No new warnings
- Determinism verified

## Test Data

Store test corpora in `tests/corpora/`:
- Valid event logs
- Various WASM tools
- Policy documents
- Patch examples

## Mock Tools for Testing

```rust
pub struct DeterministicMockTool {
    pub responses: Vec<Vec<u8>>,
}

impl Tool for DeterministicMockTool {
    fn execute(&self, _input: &[u8], _ctx: &ExecutionContext) -> ToolResult<Vec<u8>> {
        Ok(self.responses[self.counter % self.responses.len()].clone())
    }
}
```
