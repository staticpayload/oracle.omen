//! WASM sandbox with fuel and memory limits.

use oracle_omen_core::hash::Hash;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// WASM sandbox for isolated tool execution
pub struct Sandbox {
    max_fuel: u64,
    max_memory_pages: u32,
    timeout: Duration,
}

impl Sandbox {
    /// Create new sandbox
    pub fn new(max_fuel: u64, max_memory_pages: u32, timeout_ms: u64) -> Self {
        Self {
            max_fuel,
            max_memory_pages,
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    /// Create default sandbox
    pub fn default_limits() -> Self {
        Self {
            max_fuel: 1_000_000,
            max_memory_pages: 16, // 1 MB (64KB per page)
            timeout: Duration::from_secs(5),
        }
    }

    /// Execute a WASM module
    pub fn execute(&self, wasm_bytes: &[u8], input: &[u8]) -> SandboxResult<Vec<u8>> {
        // Create WASMI instance with fuel metering
        let mut engine = wasmi::Engine::default();
        let module = wasmi::Module::new(&engine, wasm_bytes)
            .map_err(|e| SandboxError::CompilationFailed(e.to_string()))?;

        // Create store with limits
        let mut store = wasmi::Store::new(&engine, Self::default_limits());
        store.limiter(|state| state.as_mut()).fuel_limit(self.max_fuel);

        // Instantiate with host imports
        let linker = wasmi::Linker::new(&engine);
        let mut instance = linker
            .instantiate(&mut store, &module)
            .and_then(|i| i.start(&mut store))
            .map_err(|e| SandboxError::InstantiationFailed(e.to_string()))?;

        // Allocate and write input
        let alloc = instance
            .get_typed_func::<i32, i32>(&mut store, "alloc")
            .ok_or(SandboxError::MissingExport("alloc".to_string()))?;

        let input_ptr = alloc
            .call(&mut store, input.len() as i32)
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        let memory = instance
            .get_memory(&mut store, 0)
            .ok_or(SandboxError::MissingMemory)?;

        memory
            .write(&mut store, input_ptr as usize, input)
            .map_err(|e| SandboxError::MemoryAccessFailed(e.to_string()))?;

        // Call main function
        let run = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "run")
            .ok_or(SandboxError::MissingExport("run".to_string()))?;

        let result_ptr = run
            .call(&mut store, input_ptr, input.len() as i32)
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        // Read output
        let mut output_data = vec
![0u8; 1024]; // Max output size
        let output_len = instance
            .get_typed_func::<i32, i32>(&mut store, "output_size")
            .ok_or(SandboxError::MissingExport("output_size".to_string()))?
            .call(&mut store, result_ptr)
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        if output_len > 0 && (output_len as usize) <= output_data.len() {
            memory
                .read(&mut store, result_ptr as usize, &mut output_data[..output_len as usize])
                .map_err(|e| SandboxError::MemoryAccessFailed(e.to_string()))?;
            output_data.truncate(output_len as usize);
        } else {
            output_data.clear();
        }

        // Get fuel consumed
        let fuel_consumed = store.fuel_consumed()
;

        Ok(ExecutionResult {
            output: output_data,
            fuel_consumed,
            success: true,
            error: None,
        })
    }

    /// Execute with timeout
    pub fn execute_with_timeout(
        &self,
        wasm_bytes: &[u8],
        input: &[u8],
    ) -> SandboxResult<Vec<u8>> {
        // For now, execute normally
        // In production, use a thread with timeout
        self.execute(wasm_bytes, input)
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::default_limits()
    }
}

/// State for store limiter
struct LimitsState {
    memory_pages: u32,
    max_pages: u32,
}

impl Default for LimitsState {
    fn default() -> Self {
        Self {
            memory_pages: 0,
            max_pages: 16,
        }
    }
}

impl wasmi::FuelLimiter for LimitsState {
    fn fuel_remaining(&mut self) -> Result<u64, wasmi::Error> {
        Ok(u64::MAX)
    }

    fn add_fuel(&mut self, fuel: u64) -> Result<(), wasmi::Error> {
        Ok(())
    }
}

impl wasmi::ResourceLimiter for LimitsState {
    fn memory_grow(&mut self, current: u32, delta: u32, max: u32) -> Result<(), wasmi::Error> {
        let new_pages = current.saturating_add(delta);
        if new_pages > self.max_pages {
            return Err(wasmi::Error::ResourceLimiterFailed(
                "Memory limit exceeded".into()
            ));
        }
        self.memory_pages = new_pages;
        Ok(())
    }

    fn table_grow(&mut self, current: u32, delta: u32, max: u32) -> Result<(), wasmi::Error> {
        let new_size = current.saturating_add(delta);
        if new_size > max {
            return Err(wasmi::Error::ResourceLimiterFailed(
                "Table limit exceeded".into()
            ));
        }
        Ok(())
    }
}

/// Result of WASM execution
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub output: Vec<u8>,
    pub fuel_consumed: u64,
    pub success: bool,
    pub error: Option<String>,
}

impl ExecutionResult {
    /// Get output hash
    pub fn output_hash(&self) -> Hash {
        Hash::from_bytes(&self.output)
    }

    /// Check if execution succeeded
    pub fn is_success(&self) -> bool {
        self.success && self.error.is_none()
    }
}

/// Sandbox errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SandboxError {
    /// Failed to compile WASM
    CompilationFailed(String),

    /// Failed to instantiate
    InstantiationFailed(String),

    /// Execution failed
    ExecutionFailed(String),

    /// Missing required export
    MissingExport(String),

    /// Missing memory
    MissingMemory,

    /// Memory access failed
    MemoryAccessFailed(String),

    /// Fuel exhausted
    FuelExhausted,

    /// Memory limit exceeded
    MemoryLimitExceeded,

    /// Timeout
    Timeout,

    /// Output too large
    OutputTooLarge,
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxError::CompilationFailed(msg) => write!(f, "Compilation failed: {}", msg),
            SandboxError::InstantiationFailed(msg) => write!(f, "Instantiation failed: {}", msg),
            SandboxError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            SandboxError::MissingExport(name) => write!(f, "Missing export: {}", name),
            SandboxError::MissingMemory => write!(f, "Missing memory export"),
            SandboxError::MemoryAccessFailed(msg) => write!(f, "Memory access failed: {}", msg),
            SandboxError::FuelExhausted => write!(f, "Fuel exhausted"),
            SandboxError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            SandboxError::Timeout => write!(f, "Execution timeout"),
            SandboxError::OutputTooLarge => write!(f, "Output too large"),
        }
    }
}

impl std::error::Error for SandboxError {}

/// Sandbox result type
pub type SandboxResult<T> = Result<T, SandboxError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new(1000000, 16, 5000);
        assert_eq!(sandbox.max_fuel, 1000000);
        assert_eq!(sandbox.max_memory_pages, 16);
    }

    #[test]
    fn test_execution_result() {
        let result = ExecutionResult {
            output: vec
![1, 2, 3],
            fuel_consumed: 100,
            success: true,
            error: None,
        };

        assert!(result.is_success()
);
        assert_ne!(result.output_hash(), Hash::zero());
    }

    // Note: Full WASM execution tests require valid WASM binaries
    // These would be added in integration tests
}
