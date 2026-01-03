# Oracle Omen v0.1.0 Release Notes

## Overview

Oracle Omen v0.1.0 is the initial release of the deterministic, auditable, capability-safe autonomous agent framework.

## Downloads

| File | BLAKE3 Hash | Size |
|------|-------------|------|
| [oracle-omen-0.1.0.tar.gz](oracle-omen-0.1.0.tar.gz) | `blake3:PLACEHOLDER` | ~500KB |
| [oracle-omen-0.1.0-bundle.tar.gz](oracle-omen-0.1.0-bundle.tar.gz) | `blake3:PLACEHOLDER` | ~502KB |

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/staticpayload/oracle.omen.git
cd oracle.omen
git checkout v0.1.0

# Build
cargo build --release

# Run tests
cargo test --all-features
```

### From Archive

```bash
# Download and extract
wget https://github.com/staticpayload/oracle.omen/releases/download/v0.1.0/oracle-omen-0.1.0.tar.gz
tar xzf oracle-omen-0.1.0.tar.gz
cd oracle-omen-0.1.0

cargo build --release
```

## Verification

```bash
# Verify BLAKE3 hash
echo "PLACEHOLDER hash" | b3sum --check

# Build should be reproducible
cargo build --release
```

## What's Included

### Crates (8)

| Crate | Description |
|-------|-------------|
| `oracle_omen_core` | Pure logic: types, hashing, events, state |
| `oracle_omen_plan` | Planning DSL and DAG compiler |
| `oracle_omen_runtime` | Tool execution and capability enforcement |
| `oracle_omen_memory` | CRDT document store with provenance |
| `oracle_omen_policy` | Policy language and evaluation engine |
| `oracle_omen_patches` | Self-evolution patch system with gates |
| `oracle_omen_wasm` | WASM sandbox with fuel limits |
| `oracle_omen_cli` | Command-line interface |

### CLI Commands

```bash
oracle-omen run <config>        # Run an agent
oracle-omen replay <run_id>     # Replay a run
oracle-omen trace <run_id>      # Show execution trace
oracle-omen diff <run_a> <run_b> # Compare runs
oracle-omen inspect <run_id>    # Inspect run details
oracle-omen capabilities <run_id> # List capability usage
oracle-omen certify <run_id>    # Certify determinism
```

### Documentation (16 files)

- [README.md](../README.md) - Complete project specification
- [docs/ARCHITECTURE.md](ARCHITECTURE.md) - System design and invariants
- [docs/EVENT_LOG.md](EVENT_LOG.md) - Event log specification
- [docs/REPLAY.md](REPLAY.md) - Replay and divergence detection
- [docs/CAPABILITIES.md](CAPABILITIES.md) - Capability system
- [docs/TOOLS.md](TOOLS.md) - Tool system
- [docs/PLANNING.md](PLANNING.md) - Planning and DAG
- [docs/MEMORY.md](MEMORY.md) - Memory CRDT
- [docs/POLICY.md](POLICY.md) - Policy language
- [docs/PATCHES.md](PATCHES.md) - Self-evolution patches
- [docs/WASM.md](WASM.md) - WASM sandbox
- [docs/CLI.md](CLI.md) - CLI reference
- [docs/SECURITY.md](SECURITY.md) - Security model
- [docs/FAILURE_MODES.md](FAILURE_MODES.md) - Failure enumeration
- [docs/DETERMINISM.md](DETERMINISM.md) - Determinism testing
- [docs/TESTING.md](TESTING.md) - Test coverage
- [docs/AUDIT_GUIDE.md](AUDIT_GUIDE.md) - Audit procedures

### Examples (8 programs)

- `echo_agent.rs` - Simple echo agent
- `tool_example.rs` - Tool capabilities
- `replay_example.rs` - Replay demonstration
- `memory_example.rs` - CRDT and provenance
- `plan_example.rs` - Planning DSL
- `policy_example.rs` - Policy evaluation
- `patch_example.rs` - Patch lifecycle
- `wasm_example.rs` - WASM sandbox

## Key Features

### Determinism

- BLAKE3 hashing for all events and state
- BTreeMap for deterministic iteration
- LogicalTime replaces system time
- Event log is source of truth
- Replay produces bit-identical results

### Capability Safety

- Zero-trust: no ambient authority
- All tools declare required capabilities
- Capability denials logged as events
- WASM sandbox for untrusted tools

### Auditability

- Every action logged with causal linkage
- Memory writes tagged with event ID
- Tool requests/responses logged
- Policy decisions logged
- Full traceability from log

### Self-Evolution

- Patches are data, not code
- Three-gate system: test, audit, approval
- Ed25519 signatures for patches
- All patch applications logged and reversible

## Known Limitations

### Not Guaranteed

- Model correctness (LLM may produce wrong outputs)
- Tool truthfulness (tools may return false data)
- Optimality (planning may not be optimal)
- Hardware-level attack mitigation

### Out of Scope

- Side-channel attacks
- Cryptographic breakthroughs (BLAKE3, Ed25519)
- Physical security

## Security

### Threat Model

Oracle Omen assumes:
- Tools may be malicious
- Inputs may be adversarial
- Host system may be compromised
- Operator may be hostile

### Mitigations

- Capability-based access control
- WASM sandbox for untrusted tools
- Resource limits (fuel, memory, timeout)
- Hash verification of all events
- Patch signing and gates

## Upgrade from Previous Versions

This is the initial release. No upgrade path needed.

## Next Release (v0.2.0)

Planned features:
- [ ] Additional tool integrations
- [ ] Performance optimizations
- [ ] Extended policy language
- [ ] Additional WASM host functions
- [ ] Web UI for inspection

## Reporting Issues

Report bugs at: https://github.com/staticpayload/oracle.omen/issues

Security issues: See SECURITY.md

## License

GPL v3. See [LICENSE](../LICENSE) file.

## Contributors

- staticpayload

## Acknowledgments

Built with:
- Rust
- BLAKE3 (blake3 crate)
- Ed25519 (ed25519-dalek crate)
- WASM (wasmi crate)
