# Audit Guide

## Purpose

This guide describes how to audit an oracle.omen agent execution. Auditing is the process of verifying that an agent's execution was faithful to its recorded event log, that all state transitions are accounted for, and that no capability violations or policy breaches occurred.

**Auditing answers these questions:**
- What did the agent do?
- Why did it do it?
- Was every action authorized?
- Can the execution be reproduced?
- Did any capabilities exceed granted permissions?
- Were any patches applied without proper gates?

## Invariants

These properties must hold for any valid oracle.omen execution:

### Event Log Invariants
1. **Sequence Continuity**: Event sequence numbers increment by exactly 1
2. **Hash Chain Integrity**: Each event's `state_hash_before` equals the previous event's `state_hash_after`
3. **Parent Linkage**: Every event (except the first) has a valid parent_id
4. **Payload Hash Consistency**: `payload_hash` equals BLAKE3(payload serialized as canonical JSON)
5. **Causal Closure**: No event references a causal_event_id that doesn't exist

### State Invariants
1. **Hash Verifiability**: `state.hash()` must return the same value when computed from same data
2. **Deterministic Reconstruction**: Replaying events from any snapshot produces identical state
3. **Memory Provenance**: Every memory write has a valid causal_event_id
4. **Capability Immutability**: Granted capabilities never change during a run

### Capability Invariants
1. **Zero Ambient Authority**: No tool executes without explicit capability grant
2. **Denial Logging**: Every capability denial is logged as an event
3. **Exhaustive Enumeration**: All required capabilities are declared by tools

### Patch Invariants
1. **Signature Validity**: All applied patches have valid Ed25519 signatures
2. **Gate Sequentiality**: Patches pass gates in order: test → audit → approval
3. **Reversibility**: All patch applications are logged and reversible

## Data Structures

### Event Record
```json
{
  "id": "run_id:sequence",
  "parent_id": "run_id:sequence-1" | null,
  "timestamp": {"run_id": u64, "sequence": u64},
  "kind": "AgentInit|Observation|ToolCall|ToolResult|...",
  "payload": { ... },
  "payload_hash": "blake3_hex",
  "state_hash_before": "blake3_hex" | null,
  "state_hash_after": "blake3_hex"
}
```

### Agent State Snapshot
```json
{
  "run_id": u64,
  "state_hash": "blake3_hex",
  "memory": {
    "documents": [
      {
        "key": "string",
        "value": { ... },
        "version": {"node_id": u64, "counter": u64},
        "causal_event": "run_id:sequence"
      }
    ]
  },
  "capabilities": ["cap:string", ...],
  "applied_patches": ["patch_id", ...]
}
```

### Tool Execution Record
```json
{
  "tool_id": "tool_name",
  "required_capabilities": ["cap:action:scope"],
  "granted_capabilities": ["cap:action:scope", ...],
  "input": { ... },
  "output": { ... } | null,
  "success": bool,
  "denial_reason": string | null,
  "fuel_consumed": u64 | null
}
```

## Execution Flow

### Normal Execution
```
1. AgentInit event
   ├─ Initialize state
   ├─ Log granted capabilities
   └─ Create initial snapshot

2. For each step:
   ├─ Agent observes context
   ├─ Observation event logged
   ├─ Agent decides action
   ├─ Policy evaluated
   ├─ Capabilities checked
   ├─ Tool invoked (or denied)
   ├─ ToolCall/ToolResult events
   └─ State updated with causal link

3. Agent completes or fails
   ├─ Final state snapshot
   └─ Final event logged
```

### Patch Application Flow
```
1. Patch proposed
   ├─ PatchCreated event
   └─ Patch stored with status=Proposed

2. Test gate
   ├─ Tests executed
   ├─ PatchTested event
   └─ Status → Tested (or Rejected)

3. Audit gate
   ├─ Policy violations checked
   ├─ Safety violations checked
   ├─ PatchAudited event
   └─ Status → Audited (or Rejected)

4. Approval gate
   ├─ Signature verified
   ├─ Signer authorization checked
   ├─ PatchApproved event
   └─ Status → Approved (or Rejected)

5. Application
   ├─ State mutations applied
   ├─ PatchApplied event
   └─ State hash updated
```

## Audit Procedures

### 1. Verify Event Log Integrity

```bash
# Verify hash chain
oracle-omen audit <run_id> --verify-hashes

# Expected output:
# ✓ Event 0: payload hash valid
# ✓ Event 1: payload hash valid, state hash chain valid
# ✓ Event 2: payload hash valid, state hash chain valid
# ...
# ✓ All hashes valid
```

**What this checks:**
- Each event's payload_hash matches BLAKE3(payload)
- Each event's state_hash_after equals next event's state_hash_before
- No missing or corrupted events

### 2. Verify Capability Compliance

```bash
# Check for capability violations
oracle-omen audit <run_id> --check-capabilities

# Expected output:
# ✓ All tool calls had required capabilities
# ✓ 0 denials logged
# OR
# ✗ Event 5: Tool 'file_write' required 'fs:write:*' but not granted
```

**What this checks:**
- Every tool call had all required capabilities granted
- All denials were properly logged
- No capability was used without being granted at init

### 3. Verify Determinism (Replay)

```bash
# Replay the run
oracle-omen replay <run_id> --verify

# Expected output:
# ✓ Replay successful
# ✓ Final state hash matches: abc123...
# OR
# ✗ Divergence detected at event 42
#   Expected: state_hash X
#   Got:      state_hash Y
```

**What this checks:**
- Replaying events produces identical final state
- No non-deterministic behavior occurred
- Tools behaved deterministically

### 4. Verify Memory Provenance

```bash
# Trace memory writes to events
oracle-omen audit <run_id> --trace-memory

# Expected output:
# memory:key_123 → Written by event 7 (ToolCall:hash_compute)
# memory:key_456 → Written by event 12 (Observation)
# ...
```

**What this checks:**
- Every memory write has a valid causal_event_id
- No orphaned data exists
- Temporal queries work correctly

### 5. Verify Patch Signatures

```bash
# Verify all applied patches
oracle-omen audit <run_id> --verify-patches

# Expected output:
# ✓ Patch 1:0 signature valid, signer authorized
# ✓ Patch 1:1 signature valid, signer authorized
# OR
# ✗ Patch 1:2: Invalid signature
```

**What this checks:**
- All applied patches have valid Ed25519 signatures
- Signers were in authorized signers list at time of application
- Patches passed all gates before application

### 6. Full Audit Report

```bash
# Generate comprehensive audit report
oracle-omen audit <run_id> --full-report > audit_report.txt

# Report includes:
# - Event log integrity
# - Capability compliance
# - Replay verification
# - Memory provenance
# - Patch verification
# - Policy decisions
# - Resource usage
```

## Failure Cases

### Event Log Corruption

**Symptoms:**
- Hash mismatch on any event
- Missing sequence number
- Broken parent linkage

**Detection:**
```bash
oracle-omen audit <run_id> --verify-hashes
```

**Recovery:**
1. Identify last good event (last successful hash verification)
2. Locate snapshot before or at that event
3. Replay from snapshot
4. If replay succeeds, corruption is isolated to post-snapshot events
5. If replay fails, snapshot may also be corrupted—use earlier snapshot

### Divergent Replay

**Symptoms:**
- `state_hash_after` doesn't match replay's computed state
- Tool returned different output than logged

**Detection:**
```bash
oracle-omen replay <run_id> --verify
```

**Root Causes:**
1. Non-deterministic tool (returned different values on replay)
2. Bug in replay engine
3. External state changed (not logged)
4. WASM tool used non-deterministic host function

**Recovery:**
1. Identify divergence point with `oracle-omen diff <run_id> <replay_id>`
2. Examine tool at divergence
3. If tool is non-deterministic, fix or mark as impure
4. If replay engine bug, fix and re-replay
5. Record divergence in audit findings

### Capability Violation

**Symptoms:**
- Tool executed without required capability
- Capability not in granted set but was used

**Detection:**
```bash
oracle-omen audit <run_id> --check-capabilities
```

**Recovery:**
1. Identify violating event
2. Determine if capability should have been granted
3. If yes: update initial capabilities, re-run
4. If no: this is a security breach—investigate further
5. Review similar runs for same vulnerability

### Missing Snapshot

**Symptoms:**
- Cannot load snapshot for replay
- Snapshot file missing or corrupted

**Recovery:**
1. Use earlier snapshot if available
2. If no earlier snapshot, replay from event 0
3. Consider snapshot retention policy

### Invalid Patch Signature

**Symptoms:**
- Applied patch has invalid Ed25519 signature
- Signer not in authorized list

**Detection:**
```bash
oracle-omen audit <run_id> --verify-patches
```

**Recovery:**
1. Identify when patch was applied
2. Create reversal patch
3. Revert state to pre-patch snapshot
4. Investigate how invalid patch was applied
5. Review approval gate for bugs

## Replay Implications

### What Replay Proves
- **Execution was deterministic**: Same inputs → same outputs
- **Event log is complete**: No hidden operations occurred
- **Tools are pure**: Tool behavior is reproducible
- **No clock dependencies**: LogicalTime is sufficient

### What Replay Does NOT Prove
- **Tool correctness**: A buggy tool will replay its bugs faithfully
- **External truth**: Tool outputs may be factually wrong
- **Model decisions**: LLM/planner choices may be suboptimal

### When Replay Is Required

1. **After any code change**: Verify new logic produces same results
2. **After external failure**: Diagnose what went wrong
3. **For compliance**: Demonstrate execution fidelity
4. **For debugging**: Isolate divergent behavior

### When Replay May Diverge (Expected)

1. **Intentional changes**: Code changes that modify behavior
2. **Different initial state**: Starting from different snapshot
3. **Different capabilities**: Granted set differs
4. **Different tools**: Tool implementation changed
5. **External inputs**: Non-deterministic external data

## Audit Checklist

For a complete audit of an oracle.omen execution:

- [ ] Event log exists and is readable
- [ ] All event payload hashes verify
- [ ] State hash chain is continuous
- [ ] No missing sequence numbers
- [ ] All parent_ids are valid
- [ ] All tool calls had required capabilities
- [ ] All capability denials are logged
- [ ] Replay produces identical final state
- [ ] All memory writes have causal_event_id
- [ ] All applied patches have valid signatures
- [ ] All patches passed test gate
- [ ] All patches passed audit gate
- [ ] All patches had authorized approval
- [ ] No policy violations occurred
- [ ] Resource limits were enforced
- [ ] Final state hash matches logged value

## Audit Evidence Preservation

To support future audits:

1. **Event Logs**: Keep indefinitely (primary audit trail)
2. **Snapshots**: Keep per retention policy (enables efficient replay)
3. **Patch Files**: Keep indefinitely (proves governance)
4. **Tool Binaries**: Keep version-controlled (reproducibility)
5. **Configuration**: Keep per run (determinism)
6. **Signed Release**: Keep hash of all artifacts (integrity)

## External Audit Interface

For third-party auditors, oracle.omen provides:

### JSON Export
```bash
oracle-omen export <run_id> --format json > run_export.json
```

### Signature Bundle
```bash
oracle-omen certify <run_id> --output bundle.tar.gz
```

Bundle contains:
- Event log (signed)
- Final state hash (signed)
- All patch signatures (verifiable)
- Determinism certification (signed)
- Capability manifest (signed)

## Security Considerations for Auditors

1. **Verify signatures on all artifacts** before trusting content
2. **Use isolated environment** for replay to prevent side-effects
3. **Keep private keys offline**—only public keys needed for verification
4. **Document chain of custody** for audit materials
5. **Report vulnerabilities responsibly** via security policy

## Reference

- [EVENT_LOG.md](EVENT_LOG.md) - Event log specification
- [REPLAY.md](REPLAY.md) - Replay engine documentation
- [CAPABILITIES.md](CAPABILITIES.md) - Capability system
- [PATCHES.md](PATCHES.md) - Patch governance
- [SECURITY.md](SECURITY.md) - Security model
