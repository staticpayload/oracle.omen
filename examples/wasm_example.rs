// Example: WASM tool execution

use oracle_omen_wasm::{compile::compile_wat, limits::ResourceLimits, sandbox::Sandbox};

// Simple WASM tool that echoes input
const ECHO_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (data (i32.const 0) "Hello from WASM!")

  (func (export "run") (param $ptr i32) (param $len i32) (result i32)
    ;; Return success
    i32.const 0
  )

  (func (export "alloc") (param $size i32) (result i32)
    (local $pages i32)
    ;; Calculate pages needed (round up)
    local.get $size
    i32.const 65535
    i32.add
    i32.const 65536
    i32.div_u
    local.tee $pages
    ;; Grow memory
    local.get $pages
    memory.grow
    ;; Return previous page count as pointer (in bytes)
    local.get $pages
    i32.const 65536
    i32.mul
  )

  (func (export "output_size") (param $result_ptr i32) (result i32)
    i32.const 16
  )
)
"#;

// WASM tool that computes hash
const HASH_WAT: &str = r#"
(module
  (memory (export "memory") 1)

  (func (export "run") (param $ptr i32) (param $len i32) (result i32)
    ;; Simple hash: just return input length as "hash"
    local.get $len
    i32.eqz
    if
      (then (return (i32.const -1)))
    end
    ;; Success
    i32.const 0
  )

  (func (export "alloc") (param $size i32) (result i32)
    local.get $size
    i32.const 65535
    i32.add
    i32.const 65536
    i32.div_u
    memory.grow
    i32.const 65536
    i32.mul
  )

  (func (export "output_size") (param $result_ptr i32) (result i32)
    i32.const 8
  )
)
"#;

fn main() {
    println!("Oracle Omen - WASM Sandbox Example");
    println!("====================================\n");

    // Compile WAT to WASM
    println!("Compiling WASM modules...");
    let echo_wasm = compile_wat(ECHO_WAT).unwrap();
    let hash_wasm = compile_wat(HASH_WAT).unwrap();
    println!("Echo module: {} bytes", echo_wasm.len());
    println!("Hash module: {} bytes\n", hash_wasm.len());

    // Create sandbox with resource limits
    let limits = ResourceLimits::minimal();
    let sandbox = Sandbox::new(
        limits.max_fuel,
        limits.max_memory_pages,
        limits.timeout_ms,
    );

    println!("Sandbox limits:");
    println!("  Fuel: {}", limits.max_fuel);
    println!("  Memory: {} pages ({} bytes)", limits.max_memory_pages, limits.max_memory_bytes());
    println!("  Timeout: {}ms\n", limits.timeout_ms);

    // Execute echo tool
    println!("Executing echo tool...");
    match sandbox.execute(&echo_wasm, b"test input") {
        Ok(result) => {
            println!("  Success: {}", result.success);
            println!("  Fuel consumed: {}", result.fuel_consumed);
            println!("  Output length: {} bytes", result.output.len());
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!();

    // Execute hash tool
    println!("Executing hash tool...");
    match sandbox.execute(&hash_wasm, b"data to hash") {
        Ok(result) => {
            println!("  Success: {}", result.success);
            println!("  Fuel consumed: {}", result.fuel_consumed);
            println!("  Output: {:?}", String::from_utf8_lossy(&result.output));
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\nWASM sandbox provides:");
    println!("  - Isolation from host system");
    println!("  - Fuel-based execution limits");
    println!("  - Memory size limits");
    println!("  - Deterministic execution");
}
