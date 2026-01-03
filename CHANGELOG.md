# Changelog

All notable changes to oracle.omen will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of oracle.omen framework
- Deterministic event logging with BLAKE3 hash chaining
- Replay engine with divergence detection
- Capability-based access control system
- Planning DSL with DAG compilation
- Memory CRDT with provenance tracking
- Policy language and evaluation engine
- Self-evolution patch system with gates
- WASM sandbox with fuel and memory limits
- CLI with run, replay, trace, diff, inspect, capabilities, and certify commands
- Comprehensive documentation (16 files)
- Example programs for all major features
- CI workflow for testing and validation

### Determinism Impact
- All hashing uses BLAKE3 with canonical JSON encoding
- All collections use BTreeMap for stable iteration order
- LogicalTime replaces system time in all critical paths
- No unseeded randomness in any execution path
- Event log is source of truth; state is derived

## [0.1.0] - 2024-01-XX

### Added
- **Core Framework**
  - `oracle_omen_core` crate with pure logic types
  - Event log schema with hash chaining
  - LogicalTime for deterministic timestamps
  - Hash type using BLAKE3
  - Agent state machine
  - Capability types (Capability, CapabilitySet)
  - Event types covering all agent operations

- **Planning System**
  - `oracle_omen_plan` crate
  - Plan DSL for declarative workflows
  - DAG compilation from plans
  - Dependency resolution and validation
  - Failure policy support (Stop, Continue, Retry, Compensate, Fallback)

- **Runtime**
  - `oracle_omen_runtime` crate
  - Tool trait with capability declaration
  - Capability enforcement before all tool execution
  - Tool registry and execution scheduler
  - Capability denial logging
  - Determinism enforcement in execution paths

- **Replay Engine**
  - Event log replay from any snapshot
  - Divergence detection with minimal diffs
  - State reconstruction verification
  - Hash chain integrity checking

- **Memory System**
  - `oracle_omen_memory` crate
  - CRDT documents with LWW semantics
  - Provenance tracking (causal_event_id)
  - Temporal state queries
  - Deterministic retrieval ordering (BTreeMap)

- **Policy Engine**
  - `oracle_omen_policy` crate
  - Policy language with rules, conditions, actions
  - Policy compiler to executable form
  - Evaluation engine with context
  - Default-deny semantics

- **Patch System**
  - `oracle_omen_patches` crate
  - Patch types for prompts, policies, routing, config, tools
  - Ed25519 signatures for patches
  - Three-gate system: test, audit, approval
  - Patch application and rollback
  - Patch store for lifecycle management

- **WASM Sandbox**
  - `oracle_omen_wasm` crate
  - WAT compilation to WASM
  - Fuel-limited execution
  - Memory page limits
  - Host function whitelisting
  - Determinism normalization

- **CLI**
  - `oracle_omen_cli` crate
  - `run` command for agent execution
  - `replay` command for replay verification
  - `trace` command for execution traces
  - `diff` command for run comparison
  - `inspect` command for run details
  - `capabilities` command for capability usage
  - `certify` command for determinism certification
  - JSON and table output formats

- **Documentation**
  - README.md with complete specification
  - ARCHITECTURE.md with system design
  - EVENT_LOG.md with event schema
  - REPLAY.md with replay procedures
  - CAPABILITIES.md with capability model
  - TOOLS.md with tool system
  - PLANNING.md with planning DSL
  - MEMORY.md with CRDT specification
  - POLICY.md with policy language
  - PATCHES.md with patch governance
  - WASM.md with sandbox details
  - CLI.md with command reference
  - SECURITY.md with threat model
  - FAILURE_MODES.md with failure enumeration
  - DETERMINISM.md with testing procedures
  - TESTING.md with test coverage
  - AUDIT_GUIDE.md with audit procedures

- **Examples**
  - echo_agent.rs: Simple echo example
  - tool_example.rs: Tool capability demonstration
  - replay_example.rs: Replay and divergence
  - memory_example.rs: CRDT and provenance
  - plan_example.rs: Planning DSL
  - policy_example.rs: Policy evaluation
  - patch_example.rs: Patch lifecycle
  - wasm_example.rs: WASM sandbox

### Changed
- None (initial release)

### Fixed
- None (initial release)

### Security
- Capability-based access control enforced at all tool boundaries
- WASM sandbox isolates untrusted tools
- All external inputs logged and hash-verified
- Patch signatures prevent unauthorized modifications

### Determinism Impact
- Critical: All state transitions logged with hash chaining
- Critical: No system time in execution paths
- Critical: BTreeMap used for all collections
- Critical: LogicalTime injected for all timestamps
- Critical: No ambient authority for any operation

## [0.0.1] - Reserved

- Placeholder for pre-release versions

---

## Versioning Policy

- **MAJOR**: Breaking changes to API, event schema, or invariants
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes, backwards compatible

## Breaking Change Policy

All breaking changes will:
1. Be documented in this CHANGELOG
2. Include migration notes in docs/MIGRATION_<version>.md
3. Maintain compatibility for at least one minor version

## Determinism Impact Categories

- **Critical**: Affects determinism guarantees, may require re-run
- **High**: May affect replay of specific operations
- **Medium**: Affects output format or presentation
- **Low**: Internal changes only, no external impact
