# Tool System

## Overview

Tools are the only way agents can interact with the world. All tools:
- Declare their capabilities
- Declare their side effects
- Declare their determinism
- Have resource bounds
- Are logged

## Tool Declaration

```rust
pub trait Tool: Send + Sync {
    fn id(&self) -> &ToolId;
    fn required_capabilities(&self) -> &[Capability];
    fn side_effects(&self) -> SideEffect;
    fn determinism(&self) -> Determinism;
    fn resource_bounds(&self) -> &ResourceBounds;
    fn execute(&self, input: &[u8], context: &ExecutionContext) -> ToolResult<Vec<u8>>;
    fn input_schema(&self) -> &str;
    fn output_schema(&self) -> &str;
}
```

## Tool Properties

### Side Effects

| Type | Description | Example |
|------|-------------|---------|
| `Pure` | No side effects | Hash computation |
| `Impure` | Has side effects | File write |

### Determinism

| Type | Description | Handling |
|------|-------------|----------|
| `Deterministic` | Same input = same output | No special handling |
| `BoundedNonDeterminism` | Uses injected time/rand | Use seeded context |
| `NonDeterministic` | External factors | Logged, not replayed |

### Resource Bounds

```rust
pub struct ResourceBounds {
    pub timeout_ms: u64,           // Maximum execution time
    pub max_memory_bytes: Option<u64>, // Memory limit
    pub max_fuel: Option<u64>,     // WASM fuel limit
}
```

## Tool Execution Flow

```
1. Capability Check -> Denied? -> Log and Stop
2. Resource Check -> Exceeded? -> Log and Stop
3. Execute Tool
4. Normalize Response
5. Log Event (with hashes)
6. Return Response
```

## Built-in Tools

### Echo Tool

Deterministic, no side effects:

```rust
impl Tool for EchoTool {
    fn id(&self) -> &ToolId {
        &ToolId::new("echo", "1.0.0")
    }

    fn required_capabilities(&self) -> &[Capability] {
        &[]
    }

    fn side_effects(&self) -> SideEffect {
        SideEffect::Pure
    }

    fn determinism(&self) -> Determinism {
        Determinism::Deterministic
    }

    fn execute(&self, input: &[u8], _context: &ExecutionContext) -> ToolResult<Vec<u8>> {
        Ok(input.to_vec())
    }
}
```

### Hash Tool

Deterministic BLAKE3 hashing:

```rust
impl Tool for HashTool {
    fn execute(&self, input: &[u8], _context: &ExecutionContext) -> ToolResult<Vec<u8>> {
        let hash = blake3::hash(input);
        Ok(hash.to_hex().into_bytes())
    }
}
```

## Response Normalization

All tool responses are normalized:

```rust
pub struct ToolResponse<T> {
    pub data: T,
    pub response_hash: Hash,  // Of normalized data
    pub metadata: ToolResponseMetadata,
}
```

Normalization ensures:
- Stable ordering for collections
- Canonical JSON representation
- Reproducible hashing

## Error Handling

Tool errors are logged:

```rust
Event {
    kind: EventKind::ToolResponse,
    payload: ToolResponsePayload {
        success: false,
        error: Some("Timeout".to_string()),
        // ...
    },
}
```

## Best Practices

1. **Declare determinism honestly** - Affects replay
2. **Use bounded resources** - Prevent runaway execution
3. **Return structured errors** - Enables better debugging
4. **Normalize outputs** - Ensures reproducibility
5. **Document capabilities** - Clear security requirements
