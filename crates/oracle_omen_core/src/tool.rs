//! Tool trait and related types for deterministic tool execution.
//!
//! Tools declare:
//! - Name and version
//! - Input/output schemas
//! - Required capabilities
//! - Side effect declaration
//! - Determinism declaration
//! - Resource bounds

#![no_std]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{fmt, time::Duration};

use crate::capability::Capability;
use crate::hash::Hash;

/// Unique tool identifier
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ToolId {
    /// Tool name
    pub name: String,

    /// Tool version (semver-like)
    pub version: String,
}

impl ToolId {
    /// Create a new tool ID
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }

    /// Create a string identifier
    #[must_use]
    pub fn as_str(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

impl Default for ToolId {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
        }
    }
}

/// Whether a tool has side effects
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SideEffect {
    /// No side effects - pure function
    Pure,

    /// Has external side effects (IO, state changes, etc.)
    Impure,
}

/// Whether a tool is deterministic
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Determinism {
    /// Fully deterministic - same input always produces same output
    Deterministic,

    /// Non-deterministic but bounded (e.g., uses time, randomness with seed)
    BoundedNonDeterminism,

    /// Fully non-deterministic
    NonDeterministic,
}

/// Resource bounds for tool execution
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResourceBounds {
    /// Maximum execution time
    pub timeout_ms: u64,

    /// Maximum memory in bytes
    pub max_memory_bytes: Option<u64>,

    /// Maximum number of operations (for WASM tools)
    pub max_fuel: Option<u64>,
}

impl ResourceBounds {
    /// Create default resource bounds
    #[must_use]
    pub fn default() -> Self {
        Self {
            timeout_ms: 30_000, // 30 seconds
            max_memory_bytes: None,
            max_fuel: None,
        }
    }

    /// Create with timeout only
    #[must_use]
    pub const fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            timeout_ms,
            max_memory_bytes: None,
            max_fuel: None,
        }
    }

    /// Get timeout as Duration
    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

/// Tool request metadata
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolRequestMetadata {
    /// Tool being called
    pub tool_id: ToolId,

    /// Request timestamp
    pub timestamp: u64,

    /// Request hash (for reproducibility)
    pub request_hash: Hash,

    /// Capabilities required
    pub required_capabilities: Vec<Capability>,
}

impl ToolRequestMetadata {
    /// Create new metadata
    #[must_use]
    pub fn new(
        tool_id: ToolId,
        timestamp: u64,
        request_hash: Hash,
        required_capabilities: Vec<Capability>,
    ) -> Self {
        Self {
            tool_id,
            timestamp,
            request_hash,
            required_capabilities,
        }
    }
}

/// Normalized tool response
///
/// Normalization ensures deterministic hashing and replay.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolResponse<T> {
    /// The response data
    pub data: T,

    /// Normalized response hash
    pub response_hash: Hash,

    /// Metadata about the response
    pub metadata: ToolResponseMetadata,
}

impl<T: serde::Serialize> ToolResponse<T> {
    /// Create a new normalized response
    pub fn new(data: T) -> Self {
        let response_hash = Hash::from_canonical(&data);
        Self {
            data,
            response_hash,
            metadata: ToolResponseMetadata::default(),
        }
    }

    /// Create with custom metadata
    pub fn with_metadata(data: T, metadata: ToolResponseMetadata) -> Self {
        let response_hash = Hash::from_canonical(&data);
        Self {
            data,
            response_hash,
            metadata,
        }
    }
}

/// Tool response metadata
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolResponseMetadata {
    /// Source of the response
    pub source: ResponseSource,

    /// Whether normalization was applied
    pub normalized: bool,

    /// Execution time in milliseconds
    pub duration_ms: u64,

    /// Additional metadata
    pub extra: Vec<(String, String)>,
}

impl ToolResponseMetadata {
    /// Create default metadata
    #[must_use]
    pub fn default() -> Self {
        Self {
            source: ResponseSource::Tool,
            normalized: true,
            duration_ms: 0,
            extra: Vec::new(),
        }
    }

    /// Create for cached response
    #[must_use]
    pub fn cached(duration_ms: u64) -> Self {
        Self {
            source: ResponseSource::Cache,
            normalized: true,
            duration_ms,
            extra: Vec::new(),
        }
    }
}

/// Source of a tool response
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResponseSource {
    /// Direct tool execution
    Tool,

    /// Cached response
    Cache,

    /// Mocked response (testing)
    Mock,

    /// Error response
    Error,
}

/// Tool trait - must be implemented by all tools
///
/// # Safety
/// Tools must not:
/// - Use global mutable state
/// - Access system time directly
/// - Use unseeded randomness
/// - Panic in normal operation
pub trait Tool: Send + Sync {
    /// Tool identifier
    fn id(&self) -> &ToolId;

    /// Required capabilities
    fn required_capabilities(&self) -> &[Capability];

    /// Side effect declaration
    fn side_effects(&self) -> SideEffect;

    /// Determinism declaration
    fn determinism(&self) -> Determinism;

    /// Resource bounds
    fn resource_bounds(&self) -> &ResourceBounds;

    /// Execute the tool
    ///
    /// # Arguments
    /// - `input`: Serialized input data
    /// - `context`: Execution context (time, run ID, etc.)
    ///
    /// # Returns
    /// Serialized output or error
    fn execute(&self, input: &[u8], context: &ExecutionContext) -> ToolResult<Vec<u8>>;

    /// Get input schema
    fn input_schema(&self) -> &str;

    /// Get output schema
    fn output_schema(&self) -> &str;
}

/// Result type for tool execution
pub type ToolResult<T> = core::result::Result<T, ToolError>;

/// Tool execution errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ToolError {
    /// Tool not found
    NotFound(String),

    /// Capability denied
    Denied { capability: String, reason: String },

    /// Timeout
    Timeout { tool: String, duration_ms: u64 },

    /// Execution failed
    ExecutionFailed { tool: String, reason: String },

    /// Invalid input
    InvalidInput { tool: String, reason: String },

    /// Output serialization failed
    SerializationFailed { tool: String, reason: String },

    /// Resource limit exceeded
    ResourceExceeded { tool: String, limit: String },

    /// Other error
    Other(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::NotFound(name) => write!(f, "Tool not found: {}", name),
            ToolError::Denied { capability, reason } => {
                write!(f, "Capability denied '{}' - {}", capability, reason)
            }
            ToolError::Timeout { tool, duration_ms } => {
                write!(f, "Tool {} timed out after {}ms", tool, duration_ms)
            }
            ToolError::ExecutionFailed { tool, reason } => {
                write!(f, "Tool {} execution failed: {}", tool, reason)
            }
            ToolError::InvalidInput { tool, reason } => {
                write!(f, "Tool {} invalid input: {}", tool, reason)
            }
            ToolError::SerializationFailed { tool, reason } => {
                write!(f, "Tool {} serialization failed: {}", tool, reason)
            }
            ToolError::ResourceExceeded { tool, limit } => {
                write!(f, "Tool {} exceeded resource limit: {}", tool, limit)
            }
            ToolError::Other(msg) => write!(f, "Tool error: {}", msg),
        }
    }
}

/// Execution context provided to tools
///
/// Contains deterministic time and other context needed for reproducible execution.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionContext {
    /// Logical timestamp
    pub logical_time: u64,

    /// Run identifier
    pub run_id: u64,

    /// Injected random seed (if needed)
    pub random_seed: Option<u64>,
}

impl ExecutionContext {
    /// Create a new execution context
    #[must_use]
    pub fn new(logical_time: u64, run_id: u64) -> Self {
        Self {
            logical_time,
            run_id,
            random_seed: None,
        }
    }

    /// Create with random seed
    #[must_use]
    pub fn with_seed(logical_time: u64, run_id: u64, random_seed: u64) -> Self {
        Self {
            logical_time,
            run_id,
            random_seed: Some(random_seed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_id() {
        let id = ToolId::new("test_tool", "1.0.0");
        assert_eq!(id.name, "test_tool");
        assert_eq!(id.version, "1.0.0");
        assert_eq!(id.as_str(), "test_tool@1.0.0");
    }

    #[test]
    fn test_resource_bounds() {
        let bounds = ResourceBounds::with_timeout(5000);
        assert_eq!(bounds.timeout_ms, 5000);
        assert_eq!(bounds.timeout(), Duration::from_millis(5000));
    }

    #[test]
    fn test_execution_context() {
        let ctx = ExecutionContext::with_seed(100, 42, 12345);
        assert_eq!(ctx.logical_time, 100);
        assert_eq!(ctx.run_id, 42);
        assert_eq!(ctx.random_seed, Some(12345));
    }
}
