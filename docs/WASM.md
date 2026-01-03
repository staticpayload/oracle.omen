# WASM Tool Sandbox

## Overview

WASM tools run in a sandbox with strict limits:
- Fuel (instruction count) limits
- Memory limits
- Whitelisted host functions only
- Deterministic execution requirements

## Why WASM?

1. **Isolation** - Tools cannot access host directly
2. **Portability** - Same binary runs anywhere
3. **Verifiability** - Binary can be inspected
4. **Limits** - Fine-grained resource control

## Limits

### Fuel

| Operation | Cost |
|-----------|-------|
| Base instruction | 1 |
| Memory access | 10 |
| Table access | 10 |
| Host call | 100 |

Default limit: 1,000,000 fuel
Maximum: 10,000,000 fuel

### Memory

- Page size: 64KB
- Default: 16 pages (1MB)
- Maximum: 64 pages (4MB)

### Output

- Default: 1MB
- Maximum: 10MB

## Required Exports

WASM tools must export:

| Function | Signature | Description |
|----------|-----------|-------------|
| `memory` | - | Linear memory |
| `run` | (i32, i32) -> i32 | Main function (ptr, len) -> status |
| `alloc` | i32 -> i32 | Allocate memory |
| `output_size` | i32 -> i32 | Get output size |

## Tool Template

```wat
(module
  (memory (export "memory") 1)

  ;; Simple echo tool
  (func (export "run") (param $ptr i32) (param $len i32) (result i32)
    ;; Return success (0)
    i32.const 0
  )

  (func (export "alloc") (param $size i32) (result i32)
    (local $ptr i32)
    ;; Simple allocation: grow memory
    local.get $size
    memory.grow
    local.tee $ptr
    i32.eqz
    (if
      (then (return (i32.const -1)))  ;; Allocation failed
    )
    ;; Return pointer (in pages, convert to bytes)
    local.get $ptr
    i32.const 65536
    i32.mul
  )

  (func (export "output_size") (param $result_ptr i32) (result i32)
    ;; Return output length
    i32.const 0  ;; No output for this example
  )
)
```

## Host Functions

Available to WASM tools:

### oracle.log(ptr, len)

Write to agent log.

```
Capability required: log
Returns: void
```

### oracle.hash(ptr, len)

Compute BLAKE3 hash.

```
Capability required: none
Returns: i32 (hash length)
```

## Compilation

```bash
# From WAT
wat2wasm tool.wat -o tool.wasm

# Optimize
wasm-opt tool.wasm -O3 -o tool.opt.wasm

# Validate
wasm-validate tool.wasm
```

## Testing

```rust
use oracle_omen_wasm::{Sandbox, ResourceLimits};

let sandbox = Sandbox::new(
    100_000,  // fuel
    4,         // 4 pages = 256KB
    1000,      // 1 second timeout
);

let result = sandbox.execute(&wasm_bytes, &input)?;
assert!(result.is_success());
```

## Determinism

WASM tools must be deterministic:
- No floating point (or use deterministic variant)
- No gethost functions (time, random)
- No external state

## Error Handling

| Error | Description | WASM Return |
|-------|-------------|-------------|
| Compilation failed | Invalid WASM | N/A |
| Instantiation failed | Import/link error | N/A |
| Execution failed | Trap/fuel exhausted | -1 |
| Timeout | Time limit exceeded | N/A |

## Security Considerations

1. **No ambient authority** - All access via host functions
2. **Capability gating** - Host functions check capabilities
3. **Resource limits** - Cannot exceed configured limits
4. **Determinism required** - Non-deterministic tools rejected

## Best Practices

1. **Minimize fuel** - More fuel = slower
2. **Pre-allocate** - Allocate once, reuse
3. **Use integers** - Avoid float for determinism
4. **Test limits** - Verify behavior at limits
5. **Log carefully** - Excessive logging is slow
