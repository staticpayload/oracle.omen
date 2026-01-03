//! Built-in tools for the runtime.

use oracle_omen_core::{
    hash::Hash,
    tool::{ResourceBounds, SideEffect, ToolError, ToolId, ToolResult},
};
use std::sync::Arc;

/// Tool registry
#[derive(Clone)]
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, Arc<dyn DynTool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn DynTool>) -> ToolResult<()> {
        let key = tool.id().as_str();
        if self.tools.contains_key(&key) {
            return Err(ToolError::NotFound(format!("Tool {} already registered", key)));
        }
        self.tools.insert(key, tool);
        Ok(())
    }

    /// Get a tool
    pub fn get(&self, id: &ToolId) -> Option<Arc<dyn DynTool>> {
        self.tools.get(&id.as_str()).cloned()
    }

    /// List all tools
    pub fn list(&self) -> Vec<ToolId> {
        self.tools.values().map(|t| t.id().clone()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamic tool trait for runtime registration
pub trait DynTool: Send + Sync {
    /// Get tool ID
    fn id(&self) -> &ToolId;

    /// Get required capabilities
    fn capabilities(&self) -> Vec<String>;

    /// Get side effect type
    fn side_effects(&self) -> SideEffect;

    /// Get resource bounds
    fn resource_bounds(&self) -> &ResourceBounds;

    /// Execute the tool
    fn execute(&self, input: &[u8], metadata: &ToolMetadata) -> ToolResult<Vec<u8>>;

    /// Get input schema
    fn input_schema(&self) -> &str;

    /// Get output schema
    fn output_schema(&self) -> &str;
}

/// Metadata passed to tool execution
#[derive(Clone, Debug)]
pub struct ToolMetadata {
    /// Logical timestamp
    pub logical_time: u64,

    /// Run ID
    pub run_id: u64,

    /// Random seed (if needed)
    pub seed: Option<u64>,
}

/// Example: Echo tool (deterministic, no side effects)
#[derive(Clone)]
pub struct EchoTool;

impl DynTool for EchoTool {
    fn id(&self) -> &ToolId {
        use std::sync::OnceLock;
        static ID: OnceLock<ToolId> = OnceLock::new();
        ID.get_or_init(|| ToolId {
            name: "echo".to_string(),
            version: "1.0.0".to_string(),
        })
    }

    fn capabilities(&self) -> Vec<String> {
        vec![]
    }

    fn side_effects(&self) -> SideEffect {
        SideEffect::Pure
    }

    fn resource_bounds(&self) -> &ResourceBounds {
        static BOUNDS: ResourceBounds = ResourceBounds::with_timeout(1000);
        &BOUNDS
    }

    fn execute(&self, input: &[u8], _metadata: &ToolMetadata) -> ToolResult<Vec<u8>> {
        Ok(input.to_vec())
    }

    fn input_schema(&self) -> &str {
        r#"{"type": "string"}"#
    }

    fn output_schema(&self) -> &str {
        r#"{"type": "string"}"#
    }
}

/// Example: Hash computation tool
#[derive(Clone)]
pub struct HashTool;

impl DynTool for HashTool {
    fn id(&self) -> &ToolId {
        use std::sync::OnceLock;
        static ID: OnceLock<ToolId> = OnceLock::new();
        ID.get_or_init(|| ToolId {
            name: "hash".to_string(),
            version: "1.0.0".to_string(),
        })
    }

    fn capabilities(&self) -> Vec<String> {
        vec![]
    }

    fn side_effects(&self) -> SideEffect {
        SideEffect::Pure
    }

    fn resource_bounds(&self) -> &ResourceBounds {
        static BOUNDS: ResourceBounds = ResourceBounds::with_timeout(1000);
        &BOUNDS
    }

    fn execute(&self, input: &[u8], _metadata: &ToolMetadata) -> ToolResult<Vec<u8>> {
        let hash = Hash::from_bytes(input);
        Ok(hash.to_hex().into_bytes())
    }

    fn input_schema(&self) -> &str {
        r#"{"type": "string"}"#
    }

    fn output_schema(&self) -> &str {
        r#"{"type": "string", "description": "BLAKE3 hex hash"}"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(EchoTool)).unwrap();
        registry.register(Arc::new(HashTool)).unwrap();

        assert_eq!(registry.list().len(), 2);
    }

    #[test]
    fn test_echo_tool() {
        let tool = EchoTool;
        let metadata = ToolMetadata {
            logical_time: 0,
            run_id: 1,
            seed: None,
        };

        let input = b"hello";
        let output = tool.execute(input, &metadata).unwrap();
        assert_eq!(output, b"hello");
    }

    #[test]
    fn test_hash_tool() {
        let tool = HashTool;
        let metadata = ToolMetadata {
            logical_time: 0,
            run_id: 1,
            seed: None,
        };

        let input = b"test";
        let output = tool.execute(input, &metadata).unwrap();
        let hash_str = String::from_utf8(output).unwrap();
        assert_eq!(hash_str.len(), 64); // BLAKE3 is 32 bytes = 64 hex chars
    }
}
