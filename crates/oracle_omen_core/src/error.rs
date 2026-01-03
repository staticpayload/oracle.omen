//! Core error types for oracle_omen.
//!
//! All errors are data. No panics in runtime paths.

#![no_std]

extern crate alloc;

use alloc::string::String;
use core::fmt;

/// Core result type
pub type Result<T> = core::result::Result<T, Error>;

/// Core error types
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Error {
    /// Event log errors
    EventLog(EventLogError),

    /// Hash errors
    Hash(HashError),

    /// State machine errors
    State(StateError),

    /// Capability errors
    Capability(CapabilityError),

    /// Tool errors
    Tool(ToolError),

    /// Serialization errors
    Serialization(SerializationError),

    /// Validation errors
    Validation(ValidationError),

    /// Generic error with message
    Message(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EventLog(e) => write!(f, "EventLog: {}", e),
            Error::Hash(e) => write!(f, "Hash: {}", e),
            Error::State(e) => write!(f, "State: {}", e),
            Error::Capability(e) => write!(f, "Capability: {}", e),
            Error::Tool(e) => write!(f, "Tool: {}", e),
            Error::Serialization(e) => write!(f, "Serialization: {}", e),
            Error::Validation(e) => write!(f, "Validation: {}", e),
            Error::Message(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Event log errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventLogError {
    /// Invalid event ID format
    InvalidEventId(String),
    /// Parent event not found
    ParentNotFound(String),
    /// Event hash mismatch
    HashMismatch { expected: String, actual: String },
    /// Corrupted log
    CorruptedLog(String),
}

impl fmt::Display for EventLogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventLogError::InvalidEventId(id) => write!(f, "Invalid event ID: {}", id),
            EventLogError::ParentNotFound(id) => write!(f, "Parent event not found: {}", id),
            EventLogError::HashMismatch { expected, actual } => {
                write!(f, "Hash mismatch: expected {}, got {}", expected, actual)
            }
            EventLogError::CorruptedLog(msg) => write!(f, "Corrupted log: {}", msg),
        }
    }
}

/// Hash errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HashError {
    /// Invalid hash format
    InvalidFormat(String),
    /// Hash computation failed
    ComputationFailed(String),
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashError::InvalidFormat(s) => write!(f, "Invalid hash format: {}", s),
            HashError::ComputationFailed(s) => write!(f, "Hash computation failed: {}", s),
        }
    }
}

/// State machine errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StateError {
    /// Invalid state transition
    InvalidTransition { from: String, to: String },
    /// Missing required state
    MissingState(String),
    /// State corruption detected
    Corrupted(String),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::InvalidTransition { from, to } => {
                write!(f, "Invalid state transition: {} -> {}", from, to)
            }
            StateError::MissingState(s) => write!(f, "Missing required state: {}", s),
            StateError::Corrupted(s) => write!(f, "State corruption: {}", s),
        }
    }
}

/// Capability errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CapabilityError {
    /// Capability not granted
    Denied { capability: String, reason: String },
    /// Invalid capability format
    InvalidFormat(String),
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapabilityError::Denied { capability, reason } => {
                write!(f, "Capability denied '{}' - {}", capability, reason)
            }
            CapabilityError::InvalidFormat(s) => write!(f, "Invalid capability format: {}", s),
        }
    }
}

/// Tool errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ToolError {
    /// Tool not found
    NotFound(String),
    /// Timeout
    Timeout { tool: String, duration_ms: u64 },
    /// Execution failed
    ExecutionFailed { tool: String, reason: String },
    /// Invalid input
    InvalidInput { tool: String, reason: String },
    /// Output serialization failed
    SerializationFailed { tool: String, reason: String },
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::NotFound(name) => write!(f, "Tool not found: {}", name),
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
        }
    }
}

/// Serialization errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SerializationError {
    /// JSON serialization failed
    Json(String),
    /// Binary serialization failed
    Binary(String),
    /// Deserialization failed
    Deserialization(String),
    /// Non-deterministic serialization detected
    NonDeterministic(String),
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::Json(s) => write!(f, "JSON serialization failed: {}", s),
            SerializationError::Binary(s) => write!(f, "Binary serialization failed: {}", s),
            SerializationError::Deserialization(s) => write!(f, "Deserialization failed: {}", s),
            SerializationError::NonDeterministic(s) => {
                write!(f, "Non-deterministic serialization: {}", s)
            }
        }
    }
}

/// Validation errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationError {
    /// Invalid value
    InvalidValue(String),
    /// Missing required field
    MissingField(String),
    /// Constraint violation
    ConstraintViolation(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidValue(s) => write!(f, "Invalid value: {}", s),
            ValidationError::MissingField(s) => write!(f, "Missing required field: {}", s),
            ValidationError::ConstraintViolation(s) => write!(f, "Constraint violation: {}", s),
        }
    }
}

// Conversion impls
impl From<EventLogError> for Error {
    fn from(e: EventLogError) -> Self {
        Error::EventLog(e)
    }
}

impl From<HashError> for Error {
    fn from(e: HashError) -> Self {
        Error::Hash(e)
    }
}

impl From<StateError> for Error {
    fn from(e: StateError) -> Self {
        Error::State(e)
    }
}

impl From<CapabilityError> for Error {
    fn from(e: CapabilityError) -> Self {
        Error::Capability(e)
    }
}

impl From<ToolError> for Error {
    fn from(e: ToolError) -> Self {
        Error::Tool(e)
    }
}

impl From<SerializationError> for Error {
    fn from(e: SerializationError) -> Self {
        Error::Serialization(e)
    }
}

impl From<ValidationError> for Error {
    fn from(e: ValidationError) -> Self {
        Error::Validation(e)
    }
}
