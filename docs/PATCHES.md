# Self-Evolution Patch System

## Overview

Agents can propose patches to their own behavior:
- Prompts
- Policies
- Routing heuristics
- Tool configurations

Patches are data, not code. They must:
- Be versioned
- Be signed
- Pass tests
- Be auditable
- Be replay-safe

## Patch Proposal

```rust
pub struct PatchProposal {
    pub patch_type: PatchType,
    pub target: String,       // What to patch
    pub patch: String,        // Patch data
    pub reasoning: String,    // Why this change
    pub test_requirements: Vec<String>,
}
```

## Patch Types

| Type | Target | Example |
|------|--------|---------|
| `Prompt` | System prompt | Update instructions |
| `Policy` | Decision policy | Change routing |
| `Routing` | Heuristic | Adjust weights |
| `Config` | Configuration | Update parameters |
| `Tools` | Tool set | Add/remove tool |

## Patch Lifecycle

```
Propose -> Sign -> Test Gate -> Audit Gate -> Apply -> Log
                                      |
                                      v
                                 Reject (with reason)
```

## Test Gate

Patches must pass tests:

```rust
fn test_gate(patch: &PatchProposal) -> GateResult {
    for test in &patch.test_requirements {
        if !run_test(test)? {
            return GateResult::Rejected {
                reason: format!("Test failed: {}", test),
            };
        }
    }
    GateResult::Accepted
}
```

## Audit Gate

Patches are reviewed:

```rust
fn audit_gate(patch: &PatchProposal) -> GateResult {
    // Check for unsafe changes
    // Verify reasoning is sound
    // Ensure compatibility with existing state
}
```

## Application

When a patch is applied:

```rust
Event {
    kind: EventKind::PatchApplied,
    payload: PatchAppliedPayload {
        patch_type: PatchType::Prompt,
        target: "system_prompt".to_string(),
        patch_hash: Hash::from_canonical(&patch),
        reasoning: "Improved clarity".to_string(),
    },
}
```

## Rejection

Rejected patches are logged:

```rust
Event {
    kind: EventKind::PatchRejected,
    payload: PatchRejectedPayload {
        patch_hash: Hash::from_canonical(&patch),
        reason: "Failed safety test".to_string(),
        stage: "test_gate".to_string(),
    },
}
```

## Replay Safety

Patches must be replay-safe:
- Deterministic application
- No external dependencies
- Version controlled
- Reversible

## Invariants

1. **Patches are data**: Never executable code
2. **All patches tested**: Test gate is mandatory
3. **All patches logged**: Full history
4. **Patches are signed**: Authenticated source
5. **Replay safe**: Can be re-applied deterministically
