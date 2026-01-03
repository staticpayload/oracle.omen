# Security Model

## Overview

Oracle Omen assumes a hostile environment. Tools are untrusted. All capabilities must be explicit and verified.

## Core Security Principles

1. **Zero Trust by Default**: No capability is granted unless explicitly given
2. **Capability-Based Security**: All actions require specific capabilities
3. **Sandboxed Execution**: Tools can run in WASM with strict limits
4. **Audit Everything**: Every capability check is logged
5. **Deterministic Verification**: All actions must be replay-identical

## Threat Model

### Untrusted Tools

Tools may:
- Attempt file access without capability
- Attempt network access without capability
- Attempt privilege escalation
- Consume excessive resources
- Return malformed output

### Mitigations

| Threat | Mitigation |
|--------|------------|
| Unauthorized file access | Capability check before execution |
| Resource exhaustion | Fuel limits, memory limits, timeouts |
| Malformed output | Response normalization and validation |
| Privilege escalation | Immutable capability set |
| Non-determinism | Tool declaration, replay verification |

## Capability System

### Format

Capabilities use `domain:action:scope` format:

```
fs:read:/tmp           # Read files in /tmp
network:http:get       # HTTP GET requests
process:exec:/bin/grep # Execute grep
env:read:PATH          # Read PATH variable
```

### Enforcement

```rust
// Before any tool execution
for required_capability in tool.required_capabilities() {
    if !granted_capabilities.has(required_capability) {
        log_capability_denied(tool, required_capability);
        return Err(CapacityError::Denied);
    }
}
```

### Capability Sources

Capabilities are granted from:
1. Configuration file
2. Policy engine evaluation
3. Explicit CLI arguments

Capabilities are **never** auto-granted.

## WASM Sandbox

### Resource Limits

| Resource | Default Limit | Maximum |
|----------|--------------|---------|
| Fuel | 1,000,000 instructions | 10,000,000 |
| Memory | 1 MB (16 pages) | 64 MB |
| Table size | 1024 elements | 4096 elements |
| Output | 1 MB | 10 MB |

### Host Function Whitelist

Only whitelisted host functions are available:

- `oracle.log` - Write to agent log
- `oracle.hash` - Compute BLAKE3 hash

No filesystem, network, or process access.

### Fuel Metering

Each WASM instruction consumes fuel. When fuel runs out:
1. Execution terminates
2. Event is logged
3. Tool returns error

## Patch Security

### Patch Signing

All patches must be signed using Ed25519:

```rust
pub struct SignedPatch {
    pub patch: Patch,
    pub signature: Signature,  // 64 bytes
    pub signer: SignerId,      // 32 bytes public key
}
```

### Patch Gates

Patches must pass three gates:

1. **Test Gate** - All tests must pass
2. **Audit Gate** - Policy and safety checks
3. **Approval Gate** - Authorized signature required

### Patch Injection Detection

Patches are scanned for:
- Prompt injection attempts
- Unsafe Rust patterns
- System command execution
- Transmute usage
- Raw pointer usage

## Event Log Security

### Hash Chaining

Each event hashes:
- Its own payload
- Previous event hash

```
event_hash = hash(payload || prev_event_hash)
```

### Tamper Detection

On replay:
```rust
if event.payload_hash != hash(event.payload) {
    return Err(EventLogError::HashMismatch);
}
```

## Policy Enforcement

### Policy Layers

1. **Static policies** - From configuration
2. **Dynamic policies** - From policy engine
3. **Runtime policies** - Capability checks

### Policy Evaluation Order

```
1. Static policy check (fastest)
2. Dynamic policy evaluation
3. Runtime capability check (most specific)
```

## Determinism and Security

Non-determinism is a security concern because:
- Cannot verify what happened
- Cannot reproduce bugs
- Cannot audit decisions

### Deterministic Requirements

1. No system time - use injected logical time
2. No unseeded randomness - use seeded RNG
3. Stable serialization - BTreeMap, sorted keys
4. No floating point in consensus - integer math only

## Failure Modes

See [FAILURE_MODES.md](FAILURE_MODES.md)

## Security Audit Checklist

- [ ] All capability checks logged
- [ ] No capabilities granted by default
- [ ] WASM limits enforced
- [ ] Patch signatures verified
- [ ] Prompt injection detection enabled
- [ ] Determinism tests passing
- [ ] Replay identity verified
- [ ] No unsafe code in critical paths
- [ ] Resource limits enforced
- [ ] Audit log append-only
