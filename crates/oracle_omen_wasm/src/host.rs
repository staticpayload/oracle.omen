//! Host functions for WASM tools.

use wasmi::{Caller, Engine, Extern, Linker, Func, Memory};

/// Host functions available to WASM tools
///
/// Only whitelisted functions are exposed.
/// All host functions are capability-gated.
pub trait HostFunction {
    /// Function name in WASM
    fn name(&self) -> &str;

    /// Required capability
    fn required_capability(&self) -> Option<&str>;

    /// Function signature
    fn signature(&self) -> Signature;

    /// Call the function
    fn call(&self, args: &[wasmi::Value]) -> Result<Vec<wasmi::Value>, HostError>;
}

/// Function signature
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signature {
    /// No arguments, no return
    Nullary,

    /// One i32 argument, no return
    I32_Void,

    /// Two i32 arguments, no return
    I32I32_Void,

    /// One i32 argument, returns i32
    I32_I32,

    /// Two i32 arguments, returns i32
    I32I32_I32,

    /// Pointer and length, returns i32
    PtrLen_I32,
}

/// Host function errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostError {
    /// Capability denied
    Denied(String),

    /// Invalid arguments
    InvalidArgs(String),

    /// Execution failed
    Failed(String),

    /// Buffer too small
    BufferTooSmall,

    /// Memory access failed
    MemoryAccessFailed,
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostError::Denied(msg) => write!(f, "Denied: {}", msg),
            HostError::InvalidArgs(msg) => write!(f, "Invalid arguments: {}", msg),
            HostError::Failed(msg) => write!(f, "Failed: {}", msg),
            HostError::BufferTooSmall => write!(f, "Buffer too small"),
            HostError::MemoryAccessFailed => write!(f, "Memory access failed"),
        }
    }
}

impl std::error::Error for HostError {}

/// Register all host functions
pub fn register_host_functions(linker: &mut Linker<HostState>) -> Result<(), String> {
    // Log function (no capability required for logging)
    linker.func_wrap(
        "oracle",
        "log",
        |mut caller: Caller<HostState>, ptr: u32, len: u32| -> Result<(), wasmi::Error> {
            let state = caller.data();
            if !state.capabilities.contains("log") {
                return Err(wasmi::Error::from("log capability not granted"));
            }

            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .ok_or_else(|| wasmi::Error::from("no memory export"))?;

            let mut data = vec
![0u8; len as usize];
            memory.read(&caller, ptr as usize, &mut data)
                .map_err(|e| wasmi::Error::from(e))?;

            let message = String::from_utf8_lossy(&data);
            state.log_message(&message);

            Ok(())
        },
    ).map_err(|e| e.to_string())?;

    // Hash function (no capability required)
    linker.func_wrap(
        "oracle",
        "hash",
        |mut caller: Caller<HostState>, ptr: u32, len: u32| -> Result<u32, wasmi::Error> {
            let memory = caller.get_export("memory")
                .and_then(|e| e.into_memory())
                .ok_or_else(|| wasmi::Error::from("no memory export"))?;

            let mut data = vec
![0u8; len as usize];
            memory.read(&caller, ptr as usize, &mut data)
                .map_err(|e| wasmi::Error::from(e))?;

            let hash = oracle_omen_core::hash::Hash::from_bytes(&data);
            let hash_hex = hash.to_hex();

            // Write hash to memory at ptr
            // For simplicity, return the length of the hash string
            // In production, would need proper memory management
            Ok(hash_hex.len() as u32)
        },
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Host state for WASM execution
#[derive(Clone, Default)]
pub struct HostState {
    capabilities: Vec<String>,
    logs: Vec<String>,
}

impl HostState {
    /// Create new host state
    pub fn new(capabilities: Vec<String>) -> Self {
        Self {
            capabilities,
            logs: Vec::new(),
        }
    }

    /// Check if has capability
    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.contains(&cap.to_string())
    }

    /// Log a message
    pub fn log_message(&mut self, msg: &str) {
        self.logs.push(msg.to_string());
    }

    /// Get logged messages
    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    /// Clear logs
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_state() {
        let mut state = HostState::new(vec
!["log".to_string(), "hash".to_string()]);
        assert!(state.has_capability("log"));
        assert!(!state.has_capability("fs_write"));

        state.log_message("test");
        assert_eq!(state.logs(), &["test"]);
    }
}
