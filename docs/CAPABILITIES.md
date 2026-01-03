# Capability System

## Overview

Capabilities control what tools an agent can use. They are:
- Immutable during execution
- Checked before every tool call
- Logged for audit

## Capability Format

```
domain:action:scope
```

### Domains

| Domain | Description | Example |
|--------|-------------|---------|
| `fs` | File system | `fs:read:/tmp` |
| `network` | Network access | `network:http:get` |
| `process` | Process execution | `process:exec:*` |
| `env` | Environment | `env:read:PATH` |

### Actions

- `read` - Read data
- `write` - Write/modify data
- `exec` - Execute code
- `delete` - Remove data

### Scopes

- `*` - Wildcard (all)
- Specific path/resource - e.g., `/tmp`, `https://api.example.com`

## Capability Set

```rust
let capabilities = CapabilitySet::new([
    Capability::new("fs:read:*"),
    Capability::new("network:http:get"),
]);

if capabilities.has(&Capability::new("fs:read:/tmp/file")) {
    // Granted
}
```

## Capability Checking

Before tool execution:

```rust
fn check_and_execute(
    tool: &Tool,
    capabilities: &CapabilitySet,
) -> ToolResult<Vec<u8>> {
    for required in tool.required_capabilities() {
        if !capabilities.has(required) {
            return Err(ToolError::Denied {
                capability: required.to_string(),
                reason: "Not granted".to_string(),
            });
        }
    }
    tool.execute(input, context)
}
```

## Capability Denial

When a tool is denied, an event is logged:

```rust
Event {
    kind: EventKind::CapabilityDenied,
    payload: CapabilityDeniedPayload {
        capability: Capability::new("fs:write:*"),
        tool_name: "file_writer".to_string(),
        reason: "Capability not granted".to_string(),
    },
    // ...
}
```

## Standard Capabilities

```rust
// File system
fs:read:*
fs:write:/tmp

// Network
network:http:get
network:https:*

// Process
process:exec:/usr/bin/grep

// Environment
env:read:PATH
env:read:HOME
```

## Best Practices

1. **Principle of least privilege**: Grant only what's needed
2. **Specific scopes**: Use specific paths instead of wildcards
3. **Audit denials**: Review denied capability events
4. **Document requirements**: Each tool declares required capabilities

## Invariants

1. **Immutable**: Capabilities cannot change during execution
2. **Explicit**: All capability checks are visible in code
3. **Logged**: All checks (grant/deny) produce events
4. **Deterministic**: Same capability set produces same grant/deny results
