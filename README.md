# oracle.omen

**Deterministic, Auditable, Capability-Safe Autonomous Agent Framework in Rust**

---

## One Paragraph Summary

oracle.omen is a framework for building autonomous agents that must be auditable years after execution. Unlike agent frameworks that prioritize convenience over correctness, oracle.oven enforces deterministic execution at every layer: event logging, state transitions, tool execution, and memory operations are all replayable. Every decision, capability check, denial, and state mutation is logged with causal linkage. Agents can propose patches to their own behavior, but patches must pass test gates, audit gates, and approval gates before application. The framework assumes hostile tools, hostile inputs, and unattended long-horizon execution.

---

## Core Guarantees

**Determinism**
- Same inputs to same agent with same logical time always produce same outputs
- All hashing uses BLAKE3 with canonical encoding
- All serialized structures use BTreeMap for stable key ordering
- No system time, unseeded randomness, or non-deterministic iteration in critical paths
- WASM tool execution is deterministic when configured

**Replayability**
- Full agent state can be reconstructed from event log and snapshots
- Replay engine detects single-bit divergences between runs
- Divergence detection produces minimal diffs identifying divergence point
- Replay is not "approximate" - it is bit-identical or explicitly different

**Capability Enforcement**
- All tool execution requires explicit capabilities granted at runtime start
- Capabilities are immutable during execution run
- Capability denials are first-class events in the log
- No ambient authority - tools cannot access host without granted capability

**Auditability**
- Every event is logged with ID, timestamp, payload, and hashes
- Causal linkage between events enables full traceability
- Memory writes are tagged with causal event ID
- Tool requests and responses are logged with hashes
- Policy decisions are logged

**Governed Self-Evolution**
- Agents can propose patches to prompts, policies, routing, or configuration
- Patches are data, not code
- Patches must pass test gate, audit gate, and approval gate
- All patch applications are logged and reversible
- Rejected patches are logged with reasons

---

## Non-Guarantees

**Model Correctness**
oracle.omen does not guarantee that the underlying model (LLM, planner, etc.) produces correct outputs. It only guarantees that whatever the model produced is faithfully recorded and reproducible.

**Tool Truthfulness**
Tools may return incorrect or malicious data. oracle.omen guarantees that tool outputs are logged and replayable, not that they are true.

**Optimality**
oracle.omen does not guarantee optimal planning or execution. It guarantees that the chosen path is the one actually taken and is reproducible.

**Performance Under Adversarial Load**
Resource limits (fuel, memory, timeout) protect against resource exhaustion but not against all adversarial inputs.

**Hardware-Level Attacks**
oracle.omen runs on commodity hardware and is subject to side-channels, Rowhammer, etc. These are out of scope.

**Cryptographic Breakthroughs**
 oracle.omen uses BLAKE3 and Ed25519. If these are broken, determinism and patch signatures are compromised.

---

## Architecture Overview

```
Intent → Policy → Plan → DAG → Runtime → Tools → Events → State
   ↓        ↓       ↓     ↓       ↓       ↓       ↑       ↑
   └────────┴───────┴─────┴───────┴───────┴───────┴──────┘
                    Memory (CRDT, Provenance)
```

**Crate Responsibilities**

| Crate | Responsibility | Side Effects |
|-------|----------------|--------------|
| oracle_omen_core | Types, hashing, event schema, state machine, capabilities, time | None (pure logic) |
| oracle_omen_plan | Planning DSL, DAG compilation, validation | None (compilation only) |
| oracle_omen_runtime | Tool execution, scheduling, capability enforcement | IO (tool calls) |
| oracle_omen_memory | CRDT storage, provenance tracking, queries | IO (persistence) |
| oracle_omen_policy | Policy language, compiler, evaluation engine | None (pure logic) |
| oracle_omen_patches | Patch lifecycle, signing, gates, application | None (data structures) |
| oracle_omen_wasm | WASM sandbox, fuel limits, host functions | IO (WASM execution) |
| oracle_omen_cli | Command-line interface, output formatting | IO (stdin/stdout) |

**Data Flow**

1. **Intent**: Agent decides to do something
2. **Policy Check**: Policy engine evaluates against current context
3. **Planning**: Intent compiled to DAG with dependencies
4. **Execution**: Runtime executes DAG nodes with capability checks
5. **Tool Calls**: Tools executed (possibly in WASM sandbox)
6. **Events**: Every step logged to event log
7. **State**: Agent state updated
8. **Memory**: Side-effects written to CRDT memory with provenance
9. **Replay**: Later, state reconstructed from events + snapshots

---

## Event Log as Source of Truth

The event log is the only authoritative record of execution. State can be derived from the log; the log cannot be derived from state.

**Log Structure**
```
Event 0 (run_id:sequence)
├── event_id: stable (run_id:sequence)
├── parent_id: null (first event)
├── timestamp: LogicalTime (run_id, sequence)
├── kind: AgentInit
├── payload_hash: Hash(payload)
├── state_hash_before: null
├── state_hash_after: Hash(initial_state)
└── payload: AgentInitPayload

Event 1
├── event_id: (run_id:1)
├── parent_id: (run_id:0)
├── timestamp: LogicalTime(run_id, 1)
├── kind: Observation
├── payload_hash: Hash(...)
├── state_hash_before: Hash(state_after_event_0)
├── state_hash_after: Hash(state_after_event_1)
└── payload: ObservationPayload
```

**Hash Chaining**
Each event's payload_hash commits to the event content. The replay engine verifies each hash during replay.

**Snapshot Boundaries**
Snapshots are taken at specific event numbers. Replay can start from any snapshot, then replay subsequent events.

**Why Logs Beat State**
- Logs enable divergence detection (state hash comparison)
- Logs enable audit (who did what when)
- Logs enable temporal queries (what was memory at event N?)
- State alone cannot answer "why" questions

---

## Deterministic Replay

**What Replay Means**
Replay reconstructs agent execution by reading the event log and re-applying events to state. If replay is successful, the final state hash matches the original run's final state hash.

**What Replay Detects**
- Non-deterministic tool behavior
- Bugs in replay engine
- Corruption in event log
- Drift in state machine logic

**What Replay Cannot Fix**
- Bugs that occurred in the original run
- Wrong tool outputs (replay will reproduce the same wrong outputs)
- External state changes (not in the event log)

**Example Replay Workflow**
```bash
# Run agent
oracle-omen run config.toml

# Get run ID from output
RUN_ID=42

# Replay to verify
oracle-omen replay $RUN_ID

# If hashes match, replay is successful
# If not, divergence point is shown
```

---

## Capability Model

**Why Ambient Authority is Forbidden**
Ambient authority (tools can access resources by default) violates zero-trust principles. It makes audit difficult and privilege escalation possible.

**How Capabilities are Declared**
Tools declare required capabilities:
```rust
fn required_capabilities(&self) -> &[Capability] {
    &[Capability::new("fs:read:*")]
}
```

**How Denials are Enforced and Logged**
```rust
if !granted_capabilities.has(&required_capability) {
    log_event(EventKind::CapabilityDenied, capability, tool);
    return Err(ToolError::Denied);
}
```

Denied capability attempts are first-class events:
```json
{
  "kind": "CapabilityDenied",
  "capability": "fs:write:*",
  "tool": "file_writer",
  "reason": "Capability not granted"
}
```

---

## Planning Model

**Intent**
Agent decides on a goal (e.g., "read file, compute hash, write result").

**DSL**
Plan expressed as declarative structure:
```rust
Plan {
    name: "process_file",
    steps: [
        PlanStep {
            id: "read",
            kind: Tool { name: "file_read", ... },
            dependencies: [],
            capabilities: ["fs:read:*"],
            ...
        },
        ],
}
```

**DAG**
Plan compiled to Directed Acyclic Graph:
```
read → hash → write
```

**Execution Guarantees**
- Topological order (dependencies satisfied)
- Capability checked before each node
- Failure policy determines continuation
- Resource limits enforced

**Failure Policies**
- `Stop`: Terminate plan
- `Continue`: Skip to next step
- `Retry`: Retry with exponential backoff
- `Compensate`: Run compensation step
- `Fallback`: Run alternative step

---

## Memory Model

**CRDT Usage**
Memory documents use LWW (Last-Writer-Wins) CRDT semantics:
```rust
Document {
    key: "user:123",
    value: StateData,
    version: BTreeMap<node_id, counter>,
    causal_event: event_id,
}
```

**Provenance**
Every write links to the event that caused it:
```rust
memory.write(key, value, causal_event_id);
```

Provenance tracking answers "why does this data exist?"

**Temporal Queries**
Query state at a specific event:
```rust
let state_at_n = store.state_at_event(event_id: 100);
```

**Deterministic Retrieval Ordering**
Memory always returns keys in sorted order (BTreeMap).

---

## Self Evolution Model

**What Can Change**
- System prompts
- Decision policies
- Routing heuristics
- Configuration values
- Tool sets

**What Cannot Change**
- Core framework types (event schema, hash algorithm)
- Capability system (granting logic)
- Determinism requirements
- Logging behavior

**Patch Lifecycle**
1. **Proposed**: Agent creates patch with reasoning
2. **Tested**: Test gate runs all tests
3. **Audited**: Audit gate checks policy and safety
4. **Approved**: Authorized signature added
5. **Applied**: Patch applied to state
6. **Rejected** (alternative): Patch rejected with reason

**Signing and Governance**
Patches are signed using Ed25519. Signers are listed in authorized signers list.

---

## WASM Sandbox Model

**Tool Isolation**
WASM tools run in isolated memory with no direct host access.

**Fuel**
Each WASM instruction consumes fuel. When fuel runs out, execution terminates.

**Memory**
Memory pages are limited (default 16 pages = 1MB, max 64 pages = 4MB).

**Determinism Normalization**
WASM tools must be deterministic. Non-deterministic host functions are not available.

---

## CLI Overview

**Full Command List**
- `oracle-omen run <config>` - Run an agent
- `oracle-omen replay <run_id>` - Replay a run
- `oracle-omen trace <run_id>` - Show execution trace
- `oracle-omen diff <run_a> <run_b>` - Compare two runs
- `oracle-omen inspect <run_id>` - Inspect run details
- `oracle-omen capabilities <run_id>` - List capability usage
- `oracle-omen certify <run_id>` - Certify determinism

**Deterministic Output Guarantees**
- Table output is sorted
- Event ordering is by sequence
- Hashes are stable hex strings
- No relative timestamps

**Example Commands**
```bash
# Run with verbose output
oracle-omen run config.toml -v

# Replay and verify
oracle-omen replay 42 --verify

# Show divergence
oracle-omen diff run_a run_b

# Export trace as JSON
oracle-omen trace 42 --output json
```

---

## Quick Start

**Build**
```bash
# Clone repository
git clone https://github.com/staticpayload/oracle.omen
cd oracle.omen

# Build
cargo build --release

# Run tests
cargo test --all-features
```

**Run Example**
```bash
cd examples
cargo run --example echo_agent
```

**Replay Example**
```bash
cargo run --example replay_example
```

**Diff Example**
```bash
# Run twice
oracle-omen run config.toml
oracle-omen run config.toml

# Diff the runs
oracle-omen diff run_a run_b
```

---

## Security Model

**Threat Assumptions**
- Tools may be malicious
- Inputs may be adversarial
- Host system may be compromised
- Operator may be hostile

**Mitigations**
- Capability-based access control
- WASM sandbox for untrusted tools
- Resource limits (fuel, memory, timeout)
- Hash verification of all logged events
- Patch signing and gates

**Known Risks**
- WASM compiler bugs (use wasmi)
- Cryptographic breaks in BLAKE3 or Ed25519
- Side-channel attacks (out of scope)
- Compromised signing keys

---

## Failure Modes

**Expected Failures**

| Failure | Detection | Response |
|---------|-----------|----------|
| Event log corruption | Hash mismatch on replay | Restore from snapshot |
| Divergent replay | State hash mismatch | Investigate tool, fix, rerun |
| Capability denied | Log entry | Agent must adapt or fail |
| Tool timeout | Timeout event | Apply failure policy |
| Fuel exhausted | Fuel limit reached | Terminate WASM tool |
| Memory limit | Allocation fails | Terminate execution |
| Patch test failure | Test gate returns false | Patch rejected |
| Snapshot missing | Cannot load file | Use earlier snapshot or replay from start |

**How oracle.omen Exposes Them**
All failures are logged as events with structured data.

**Operator Response**
1. Check event log for failure type
2. Consult FAILURE_MODES.md for recovery procedure
3. Apply corrective action
4. Re-run from last known good state

---

## Repository Layout

```
oracle.omen/
├── Cargo.toml              # Workspace configuration
├── LICENSE                  # GPL v3
├── README.md                # This file
├── CHANGELOG.md             # Version history
├── docs/                    # Complete documentation
│   ├── ARCHITECTURE.md      # System design and invariants
│   ├── EVENT_LOG.md         # Event log specification
│   ├── REPLAY.md            # Replay and divergence
│   ├── CAPABILITIES.md      # Capability system
│   ├── TOOLS.md             # Tool system
│   ├── PLANNING.md          # Planning and DAG
│   ├── MEMORY.md            # Memory CRDT
│   ├── POLICY.md            # Policy language
│   ├── PATCHES.md           # Self-evolution
│   ├── WASM.md              # WASM sandbox
│   ├── CLI.md               # CLI reference
│   ├── SECURITY.md          # Security model
│   ├── FAILURE_MODES.md      # Failure enumeration
│   ├── DETERMINISM.md       # Determinism testing
│   ├── TESTING.md           # Test coverage
│   └── AUDIT_GUIDE.md       # Audit procedures
├── crates/                  # Source code
│   ├── oracle_omen_core/     # Pure logic
│   ├── oracle_omen_plan/     # Planning
│   ├── oracle_omen_runtime/  # Runtime
│   ├── oracle_omen_memory/   # Memory
│   ├── oracle_omen_policy/   # Policy
│   ├── oracle_omen_patches/  # Patches
│   ├── oracle_omen_wasm/     # WASM
│   └── oracle_omen_cli/      # CLI
├── examples/                 # Example programs
│   ├── echo_agent.rs
│   ├── tool_example.rs
│   ├── replay_example.rs
│   ├── memory_example.rs
│   ├── plan_example.rs
│   ├── policy_example.rs
│   ├── patch_example.rs
│   └── wasm_example.rs
└── .github/workflows/       # CI
    └── ci.yml
```

---

## Development Workflow

**Tests**
- Unit tests in each module
- Integration tests in `tests/`
- Property-based tests with proptest
- Fuzzing targets in `fuzz/`

**CI**
- Runs on every push and PR
- Checks formatting, clippy, tests
- Runs on multiple OS (Ubuntu, macOS, Windows)

**Formatting**
```bash
cargo fmt --all
```

**Determinism Certification**
```bash
# Run determinism tests
cargo test --test determinism

# Run replay identity tests
cargo test --test replay_identity

# Cross-platform verification
cargo test --all-features --target x86_64-unknown-linux-gnu
cargo test --all-features --target aarch64-unknown-linux-gnu
```

---

## Versioning and Stability

**Semver Policy**
- MAJOR: Breaking changes to API, event schema, or invariants
- MINOR: New features, backwards compatible
- PATCH: Bug fixes, backwards compatible

**Breaking Change Rules**
- Documented in CHANGELOG.md
- Migration notes provided
- Old version supported for at least one minor version

**Migration Expectations**
- Event log format changes include migration tool
- Breaking API changes have compatibility shim
- Documented deprecation period

---

## License and Governance

**License**
GPL v3. See LICENSE file.

**Contribution Rules**
- All code must pass tests and clippy
- All changes must include tests
- All changes must update docs
- Sign off commits (DCO)

**Code of Responsibility**
- Security: Report vulnerabilities responsibly
- Correctness: Fix bugs before features
- Determinism: Never compromise determinism for convenience
- Auditability: If behavior matters, it must be logged

---

## For More Information

- [Documentation](docs/)
- [Examples](examples/)
- [Contributing](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)

**Logs are truth. Replay is judge. Types are law.**
