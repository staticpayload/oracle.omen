//! Host functions for WASM tools.

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

/// Register all host functions
///
/// This is a placeholder for full host function registration.
/// The actual implementation requires wasmi 0.35 API compatibility
/// with the Linker and Caller types.
pub fn register_host_functions(
    _linker: &mut HostState,
) -> Result<(), String> {
    // Placeholder for host function registration
    // Full implementation requires wasmi 0.35 API compatibility
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_state() {
        let state = HostState::new(vec!["log".to_string(), "hash".to_string()]);
        assert!(state.has_capability("log"));
        assert!(!state.has_capability("fs_write"));
    }

    #[test]
    fn test_host_logging() {
        let mut state = HostState::new(vec!["log".to_string()]);
        state.log_message("test");
        assert_eq!(state.logs(), &["test"]);
    }
}
