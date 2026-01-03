# Determinism Testing and Certification

## Overview

Determinism is the foundation of replay and auditability. All components must be certified deterministic.

## Testing Requirements

### Replay Identity Test

The same event log must produce the same final state across:

1. Different runs
2. Different platforms (x86_64, ARM64)
3. Different compilations

```rust
#[test]
fn test_replay_identity() {
    let log = create_test_log();
    let state1 = replay_log(&log);
    let state2 = replay_log(&log);
    assert_eq!(state1.hash(), state2.hash());
}
```

### Divergence Detection Test

A single bit change must be detected:

```rust
#[test]
fn test_divergence_detection() {
    let log1 = create_test_log();
    let mut log2 = log1.clone();
    flip_bit(&mut log2);

    let divergences = compare_logs(&log1, &log2);
    assert_eq!(divergences.len(), 1);
}
```

### Cross-Platform Test

Same inputs produce same outputs on different platforms:

```rust
#[test]
fn test_cross_platform_determinism() {
    let seed = 42;
    let input = b"test";

    let hash1 = hash_with_seed(seed, input);
    let hash2 = hash_with_seed(seed, input);
    assert_eq!(hash1, hash2);
}
```

## Certification Process

### 1. Unit Tests

Every module must have:
- Hash stability tests
- Serialization tests
- Ordering tests

### 2. Integration Tests

- Full replay test
- Tool execution test
- Policy evaluation test

### 3. Property Tests

Using proptest:

```rust
proptest! {
    #[test]
    fn prop_hash_stable(val: Vec<u8>) {
        let h1 = Hash::from_bytes(&val);
        let h2 = Hash::from_bytes(&val);
        prop_assert_eq!(h1, h2);
    }
}
```

### 4. Fuzzing

```bash
cargo fuzz run event_log_parse
cargo fuzz run hash_compute
```

## Determinism Checklist

- [ ] No HashMap iteration in serialized output
- [ ] No system time (use LogicalTime)
- [ ] No unseeded randomness
- [ ] No floating point in consensus paths
- [ ] BTreeMap for all serialized maps
- [ ] Stable field ordering in structs
- [ ] Explicit ordering for Vec where it matters

## Non-Determinism Detection

The framework actively prevents:

| Source | Prevention |
|--------|------------|
| System time | LogicalTime injection |
| Randomness | Seeded RNG in context |
| HashMap | Compile-time lint |
| Iteration order | BTreeMap enforced |
| Float | Isolated to non-critical paths |

## Certification Matrix

| Component | Unit | Integration | Property | Fuzz | Cross-Platform |
|-----------|------|-------------|----------|------|---------------|
| Event Log | ✅ | ✅ | ✅ | ✅ | ✅ |
| Hash | ✅ | ✅ | ✅ | ✅ | ✅ |
| State Machine | ✅ | ✅ | ✅ | | ✅ |
| Tools | ✅ | ✅ | | | |
| Memory | ✅ | ✅ | ✅ | | ✅ |
| Policy | ✅ | ✅ | | | |
| Patches | ✅ | ✅ | | | |
| WASM | ✅ | ✅ | | ✅ | |

## Running Tests

```bash
# All tests
cargo test

# Determinism tests
cargo test --test determinism

# Property tests
cargo test --test proptests

# Fuzz (requires nightly)
cargo +nightly fuzz run ...

# Cross-platform (run on multiple platforms)
cargo test --all-features
```

## CI Requirements

CI must run:
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test --all-features`
4. Determinism verification
5. Cross-platform builds (x86_64, ARM64)

## Failure on Non-Determinism

Any test that detects non-determinism MUST FAIL.

Non-determinism is a bug.
