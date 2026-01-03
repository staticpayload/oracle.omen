# Policy Language

## Overview

The policy language governs what an agent may do. Policies are declarative rules that are compiled and evaluated at runtime.

## Policy Document

```toml
# Example policy file
name = "agent_policy"
version = "1.0.0"

[[rules]]
name = "allow_file_read"
kind = "Capability"

[rules.condition]
type = "HasCapability"
capability = "fs:read"

[rules.action]
type = "Allow"
```

## Rule Kinds

| Kind | Description | Example |
|------|-------------|---------|
| `Capability` | Governs capability requests | Can agent request fs:read? |
| `Tool` | Governs tool execution | Can tool run? |
| `Memory` | Governs memory access | Can read/write memory key? |
| `Patch` | Governs self-modification | Can patch prompt? |
| `Resource` | Governs resource usage | Limits, quotas |

## Conditions

### HasCapability

```json
{
  "type": "HasCapability",
  "capability": "fs:read:/tmp"
}
```

### And/Or/Not

```json
{
  "type": "And",
  "conditions": [
    {"type": "HasCapability", "capability": "fs:read"},
    {"type": "HasCapability", "capability": "network:https:get"}
  ]
}
```

### Compare

```json
{
  "type": "Compare",
  "field": "iterations",
  "op": "Less",
  "value": {"type": "Integer", "value": 1000}
}
```

## Actions

| Action | Description |
|--------|-------------|
| `Allow` | Permit the operation |
| `Deny` | Reject with reason |
| `AllowModified` | Allow with modifications |
| `RequireApproval` | Need approval |
| `Log` | Log and continue |

## Evaluation

Policies are evaluated in order:
1. First matching rule wins
2. Explicit deny overrides allow
3. Default deny if no match

## Example Policies

### Minimal (Allow Nothing)

```toml
name = "minimal"
version = "1.0.0"

[[rules]]
name = "default_deny"
kind = "Capability"

[rules.condition]
type = "False"

[rules.action]
type = "Deny"
reason = "No capabilities granted"
```

### Read-Only Agent

```toml
name = "read_only"
version = "1.0.0"

[[rules]]
name = "allow_read"
kind = "Capability"

[rules.condition]
type = "HasCapability"
capability = "fs:read:*"

[rules.action]
type = "Allow"

[[rules]]
name = "deny_write"
kind = "Capability"

[rules.condition]
type = "HasCapability"
capability = "fs:write:*"

[rules.action]
type = "Deny"
reason = "Write operations not allowed"
```

### Resource Limited

```toml
name = "resource_limited"
version = "1.0.0"

[[rules]]
name = "limit_iterations"
kind = "Resource"

[rules.condition]
type = "Compare"
field = "iterations"
op = "Less"
value = {type = "Integer", value = 100}

[rules.action]
type = "Allow"
```

## Policy Composition

Multiple policies can be loaded:
```
1. Base policy (framework defaults)
2. Org policy (organizational rules)
3. Agent policy (agent-specific)
4. Run policy (per-run overrides)

Later policies override earlier ones.
```

## Invariants

1. **Deterministic evaluation** - Same policy + context = same result
2. **No side effects** - Evaluation does not mutate state
3. **Complete logging** - All policy decisions logged
4. **Explicit deny** - Default is deny, not allow
