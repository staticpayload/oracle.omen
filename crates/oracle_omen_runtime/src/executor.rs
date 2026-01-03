//! DAG executor - runs compiled DAGs with capability enforcement.

use oracle_omen_core::{capability::CapabilitySet, error::Error};
use oracle_omen_plan::dag::Dag;
use std::collections::HashMap;
use std::sync::Arc;

/// Execution result
pub type ExecResult<T> = Result<T, ExecError>;

/// Execution errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecError {
    /// Capability denied
    CapabilityDenied { capability: String, reason: String },

    /// Tool not found
    ToolNotFound(String),

    /// Timeout
    Timeout { node: String, duration_ms: u64 },

    /// Execution failed
    Failed { node: String, reason: String },

    /// Resource limit exceeded
    ResourceExceeded { node: String, limit: String },

    /// Invalid state
    InvalidState(String),
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::CapabilityDenied { capability, reason } => {
                write!(f, "Capability denied '{}': {}", capability, reason)
            }
            ExecError::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            ExecError::Timeout { node, duration_ms } => {
                write!(f, "Node {} timed out after {}ms", node, duration_ms)
            }
            ExecError::Failed { node, reason } => {
                write!(f, "Node {} failed: {}", node, reason)
            }
            ExecError::ResourceExceeded { node, limit } => {
                write!(f, "Node {} exceeded limit: {}", node, limit)
            }
            ExecError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for ExecError {}

/// Execution state
#[derive(Clone, Debug)]
pub struct ExecState {
    /// Completed nodes
    pub completed: Vec<String>,

    /// Failed nodes
    pub failed: Vec<String>,

    /// Current node being executed
    pub current: Option<String>,

    /// Node results
    pub results: HashMap<String, NodeResult>,
}

impl ExecState {
    /// Create new execution state
    pub fn new() -> Self {
        Self {
            completed: Vec::new(),
            failed: Vec::new(),
            current: None,
            results: HashMap::new(),
        }
    }

    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        !self.completed.is_empty() && self.current.is_none()
    }

    /// Check if execution failed
    pub fn has_failed(&self) -> bool {
        !self.failed.is_empty()
    }
}

impl Default for ExecState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a node execution
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeResult {
    /// Node ID
    pub node_id: String,

    /// Success status
    pub success: bool,

    /// Output data (JSON string)
    pub output: Option<String>,

    /// Error message if failed
    pub error: Option<String>,

    /// Execution time in ms
    pub duration_ms: u64,
}

impl NodeResult {
    /// Create a successful result
    pub fn success(node_id: impl Into<String>, output: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            node_id: node_id.into(),
            success: true,
            output: Some(output.into()),
            error: None,
            duration_ms,
        }
    }

    /// Create a failed result
    pub fn failure(node_id: impl Into<String>, error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            node_id: node_id.into(),
            success: false,
            output: None,
            error: Some(error.into()),
            duration_ms,
        }
    }
}

/// DAG executor
pub struct DagExecutor {
    /// Granted capabilities
    capabilities: CapabilitySet,

    /// Execution state
    state: ExecState,
}

impl DagExecutor {
    /// Create a new executor
    pub fn new(capabilities: CapabilitySet) -> Self {
        Self {
            capabilities,
            state: ExecState::new(),
        }
    }

    /// Execute a DAG
    pub async fn execute(&mut self, _dag: &Dag) -> ExecResult<ExecState> {
        // Get topological order
        // let order = dag.topological_order().map_err(|e| ExecError::InvalidState(e.to_string()))?;

        // Execute each node
        // for node_id in order {
        //     self.execute_node(dag, &node_id).await?;
        // }

        Ok(self.state.clone())
    }

    /// Execute a single node
    async fn execute_node(&mut self, _dag: &Dag, _node_id: &str) -> ExecResult<()> {
        // Check capabilities
        // Check dependencies
        // Execute
        // Record result
        Ok(())
    }

    /// Get current execution state
    pub fn state(&self) -> &ExecState {
        &self.state
    }

    /// Get granted capabilities
    pub fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }
}

/// Tool registry for execution
pub trait ToolRegistry: Send + Sync {
    /// Check if a tool exists
    fn has_tool(&self, name: &str) -> bool;

    /// Get tool
    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
}

/// Tool trait for runtime execution
pub trait Tool: Send + Sync {
    /// Get tool name
    fn name(&self) -> &str;

    /// Get required capabilities
    fn capabilities(&self) -> Vec<Capability>;

    /// Execute the tool
    fn execute(&self, input: &[u8]) -> ExecResult<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_state() {
        let state = ExecState::new();
        assert!(!state.is_complete());
        assert!(!state.has_failed());
    }

    #[test]
    fn test_node_result() {
        let result = NodeResult::success("test", "{\"value\": 42}", 100);
        assert!(result.success);
        assert_eq!(result.output, Some("{\"value\": 42}".to_string()));
    }

    #[test]
    fn test_executor_creation() {
        let executor = DagExecutor::new(CapabilitySet::empty());
        assert_eq!(executor.capabilities().len(), 0);
    }
}
