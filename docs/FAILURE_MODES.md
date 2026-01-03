# Failure Modes

## Overview

This document enumerates all known failure modes, their detection, recovery, and prevention.

## Event Log Failures

### Corrupted Log

**Detection**:
- Event sequence mismatch
- Hash verification failure
- Parent event not found

**Recovery**:
1. Restore from last valid snapshot
2. Re-run events from snapshot position
3. If corruption at snapshot, restore from earlier snapshot

**Prevention**:
- Atomic writes (write to temp, then rename)
- Hash verification after write
- Regular snapshots

### Missing Parent Event

**Detection**:
- `parent_id` references non-existent event

**Recovery**:
- Reject event with missing parent
- Request resend or rebuild from source

**Prevention**:
- Verify parent exists before append
- Causal ordering validation

## State Failures

### State Divergence

**Detection**:
- Different hash on replay
- Divergence point identified by replay engine

**Recovery**:
1. Identify divergence point using diff tool
2. Find tool that caused divergence
3. Fix tool determinism issue
4. Re-run from before divergence

**Prevention**:
- All tools must declare determinism level
- Non-deterministic tools use seeded RNG
- Regular replay verification

### State Corruption

**Detection**:
- State hash mismatch
- BTreeMap invariant violation

**Recovery**:
- Rebuild from event log
- If log also corrupted, use snapshot

**Prevention**:
- State transitions are pure functions
- No direct state mutation outside state machine

## Tool Failures

### Timeout

**Detection**:
- Tool execution exceeds timeout
- Fuel exhausted (WASM)

**Recovery**:
- Log timeout event
- Continue based on failure policy

**Prevention**:
- Reasonable timeout defaults
- Resource pre-allocation checks

### Resource Exhaustion

**Detection**:
- Memory limit exceeded
- Fuel limit exceeded
- Table limit exceeded

**Recovery**:
- Terminate tool execution
- Log resource exhaustion
- Apply failure policy

**Prevention**:
- Resource limits enforced by runtime
- WASM sandbox for untrusted tools

### Tool Panic

**Detection**:
- Tool returns error
- WASM trap

**Recovery**:
- Never panic in runtime
- Convert panics to errors
- Log and apply failure policy

**Prevention**:
- Tool validation before registration
- WASM sandbox isolation

### Malformed Output

**Detection**:
- Output validation fails
- JSON parse error
- Schema mismatch

**Recovery**:
- Log error event
- Apply failure policy

**Prevention**:
- Response normalization
- Schema validation
- Size limits

## Capability Failures

### Capability Denied

**Detection**:
- Capability check fails
- Logged as `CapabilityDenied` event

**Recovery**:
- Event is logged
- Tool does not execute
- Agent can retry with different approach

**Prevention**:
- Grant required capabilities
- Policy review

### Capability Escalation

**Detection**:
- Audit shows tool used without capability
- Impossible if runtime is correct

**Recovery**:
- N/A - This indicates security bug

**Prevention**:
- Immutable capability set
- All checks go through same code path
- Comprehensive tests

## Memory Failures

### CRDT Merge Conflict

**Detection**:
- Different values for same key at different events
- LWW semantics resolve automatically

**Recovery**:
- Automatic (LWW)
- If LWW is wrong, manual intervention needed

**Prevention**:
- Single writer per key when possible
- Event-based ordering

### Memory Exhaustion

**Detection**:
- Configured limit exceeded

**Recovery**:
- Reject write operation
- Log event

**Prevention**:
- Configurable limits
- Automatic cleanup of old data

## Patch Failures

### Test Gate Failure

**Detection**:
- Test returns fail
- Test times out

**Recovery**:
- Patch rejected
- Failure reason logged

**Prevention**:
- Comprehensive test suite
- Realistic test data

### Audit Gate Failure

**Detection**:
- Policy denies patch
- Safety check fails

**Recovery**:
- Patch rejected
- Reason logged

**Prevention**:
- Clear policy documentation
- Pre-flight validation

### Signature Verification Failure

**Detection**:
- Signature invalid
- Signer not authorized

**Recovery**:
- Patch rejected
- Security event logged

**Prevention**:
- Secure key storage
- Authorized signer list

### Rollback Failure

**Detection**:
- Cannot restore previous state
- Rollback data corrupted

**Recovery**:
- Replay event log to pre-patch state
- Mark system as needing manual review

**Prevention**:
- Store complete rollback data
- Verify rollback data integrity

## Replay Failures

### Verification Failed

**Detection**:
- Final state hash doesn't match
- Event hash mismatch

**Recovery**:
1. Identify divergence point
2. Investigate non-determinism
3. Fix and re-run

**Prevention**:
- Determinism enforcement
- Comprehensive testing

### Snapshot Missing

**Detection**:
- Cannot load referenced snapshot

**Recovery**:
- Use earlier snapshot
- Replay from beginning

**Prevention**:
- Snapshot backup
- Multiple snapshot strategy

## Network Failures (Optional)

### Tool Network Access

**Detection**:
- Connection timeout
- DNS failure
- Connection refused

**Recovery**:
- Based on tool failure policy
- Retry with exponential backoff
- Give up after max retries

**Prevention**:
- Reasonable timeouts
- Graceful degradation

## WASM Failures

### Compilation Failed

**Detection**:
- WAT parse error
- WASM validation error

**Recovery**:
- Tool cannot be used
- Error logged

**Prevention**:
- Pre-compile tools
- Validation at registration

### Instantiation Failed

**Detection**:
- Import not found
- Memory/table limit exceeded

**Recovery**:
- Tool execution fails
- Event logged

**Prevention**:
- Host function whitelisting
- Limit validation

### Fuel Exhausted

**Detection**:
- Fuel counter reaches limit

**Recovery**:
- Execution terminates
- Event logged
- Output so far is discarded

**Prevention**:
- Appropriate fuel limits
- Fuel cost estimation

## System Failures

### Out of Memory

**Detection**:
- Allocation fails
- OS OOM killer

**Recovery**:
- Process terminated
- Recovery from last log/snapshot

**Prevention**:
- Memory limits per run
- Monitoring and alerting

### Disk Full

**Detection**:
- Write fails
- ENOSPC

**Recovery**:
- Log append fails
- Run cannot continue safely

**Prevention**:
- Disk space monitoring
- Automatic log rotation
- Cleanup of old runs

### Process Crash

**Detection**:
- Signal received
- Unexpected exit

**Recovery**:
- Last event may be incomplete
- Recovery from last complete event

**Prevention**:
- Signal handlers
- Graceful shutdown
- Atomic writes

## Failure Severity Levels

| Severity | Description | Action |
|----------|-------------|--------|
| Critical | System cannot continue | Immediate intervention |
| High | Functionality severely impacted | Prompt investigation |
| Medium | Degraded but operational | Scheduled investigation |
| Low | Cosmetic or minor | Log and monitor |

## Monitoring and Alerting

### Metrics to Track

- Event log write failures
- Replay divergences
- Capability denials
- Tool timeouts
- Resource exhaustion
- Patch rejections
- WASM fuel exhaustion

### Alert Thresholds

- Any critical failure
- >10 high failures/hour
- >100 medium failures/hour
- Replay divergence (always alert)
