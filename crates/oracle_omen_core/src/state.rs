//! State machine types for agents.
//!
//! Agents are pure state machines:
//! Input: prior state, observation, tool responses, injected deterministic context
//! Output: next state, planned actions, patch proposals

#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::fmt;

use crate::hash::Hash;

/// Agent state - typed container for agent data
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentState {
    /// State version - increments on each transition
    pub version: u64,

    /// State data keyed by domain
    pub data: BTreeMap<String, StateData>,

    /// Hash of this state
    pub state_hash: Hash,
}

impl AgentState {
    /// Create initial empty state
    #[must_use]
    pub fn initial() -> Self {
        Self {
            version: 0,
            data: BTreeMap::new(),
            state_hash: Hash::zero(),
        }
    }

    /// Create state with run ID
    #[must_use]
    pub fn with_run_id(run_id: u64) -> Self {
        let mut state = Self::initial();
        state.data.insert(
            "system".to_string(),
            StateData::Value(StateValue::U64(run_id)),
        );
        state.rehash();
        state
    }

    /// Get state data for a domain
    #[must_use]
    pub fn get(&self, domain: &str) -> Option<&StateData> {
        self.data.get(domain)
    }

    /// Set state data for a domain
    pub fn set(&mut self, domain: impl Into<String>, data: StateData) {
        self.data.insert(domain.into(), data);
        self.version += 1;
        self.rehash();
    }

    /// Remove state data for a domain
    pub fn remove(&mut self, domain: &str) -> Option<StateData> {
        let result = self.data.remove(domain);
        if result.is_some() {
            self.version += 1;
            self.rehash();
        }
        result
    }

    /// Compute and store the state hash
    fn rehash(&mut self) {
        self.state_hash = Hash::from_canonical(&self.data);
    }

    /// Check if this state matches another by hash
    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        self.state_hash == other.state_hash
    }

    /// Get the state hash
    #[must_use]
    pub const fn hash(&self) -> Hash {
        self.state_hash
    }

    /// Get the version
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::initial()
    }
}

/// State data - typed values for agent state
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StateData {
    /// Single value
    Value(StateValue),

    /// Map of values
    Map(BTreeMap<String, StateValue>),

    /// List of values
    Vec(Vec<StateValue>),
}

/// Typed state value
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StateValue {
    /// String value
    String(String),

    /// Boolean value
    Bool(bool),

    /// Unsigned 64-bit integer
    U64(u64),

    /// Signed 64-bit integer
    I64(i64),

    /// Float (use carefully in consensus paths)
    F64(f64),

    /// Bytes
    Bytes(Vec<u8>),

    /// Hash reference
    Hash(Hash),

    /// Null/none
    None,
}

impl StateValue {
    /// Get type name
    #[must_use]
    pub fn type_name(&self) -> &str {
        match self {
            StateValue::String(_) => "string",
            StateValue::Bool(_) => "bool",
            StateValue::U64(_) => "u64",
            StateValue::I64(_) => "i64",
            StateValue::F64(_) => "f64",
            StateValue::Bytes(_) => "bytes",
            StateValue::Hash(_) => "hash",
            StateValue::None => "none",
        }
    }
}

impl From<String> for StateValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for StateValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<bool> for StateValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<u64> for StateValue {
    fn from(n: u64) -> Self {
        Self::U64(n)
    }
}

impl From<i64> for StateValue {
    fn from(n: i64) -> Self {
        Self::I64(n)
    }
}

impl From<Hash> for StateValue {
    fn from(h: Hash) -> Self {
        Self::Hash(h)
    }
}

/// State transition result
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateTransition {
    /// Previous state hash
    pub from_hash: Hash,

    /// New state
    pub to_state: AgentState,

    /// Event that caused the transition
    pub event_hash: Hash,

    /// Transition hash
    pub transition_hash: Hash,
}

impl StateTransition {
    /// Create a new state transition
    #[must_use]
    pub fn new(from_hash: Hash, to_state: AgentState, event_hash: Hash) -> Self {
        let transition_hash = crate::hash::transition_hash(from_hash, event_hash, to_state.hash());
        Self {
            from_hash,
            to_state,
            event_hash,
            transition_hash,
        }
    }

    /// Get the transition hash
    #[must_use]
    pub const fn hash(&self) -> Hash {
        self.transition_hash
    }
}

/// Observation - input from environment
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Observation {
    /// Observation kind/type
    pub kind: String,

    /// Observation data
    pub data: BTreeMap<String, StateValue>,

    /// Observation timestamp (logical)
    pub logical_time: u64,
}

impl Observation {
    /// Create a new observation
    #[must_use]
    pub fn new(kind: impl Into<String>, logical_time: u64) -> Self {
        Self {
            kind: kind.into(),
            data: BTreeMap::new(),
            logical_time,
        }
    }

    /// Add data field
    pub fn with_field(mut self, key: impl Into<String>, value: StateValue) -> Self {
        self.data.insert(key.into(), value);
        self
    }
}

/// Decision - output from agent
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Decision {
    /// No action - observe only
    None,

    /// Request tool execution
    ToolCall(ToolCall),

    /// Propose a self-patch
    PatchProposal(PatchProposal),

    /// Multiple actions
    Multiple(Vec<Decision>),
}

/// Tool call decision
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    /// Tool to call
    pub tool_name: String,

    /// Tool version
    pub tool_version: String,

    /// Input payload (JSON)
    pub input: String,

    /// Required capabilities
    pub capabilities: Vec<String>,
}

impl ToolCall {
    /// Create a new tool call
    #[must_use]
    pub fn new(
        tool_name: impl Into<String>,
        tool_version: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            tool_version: tool_version.into(),
            input: input.into(),
            capabilities: Vec::new(),
        }
    }

    /// Add required capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
}

/// Patch proposal for self-evolution
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PatchProposal {
    /// Patch type
    pub patch_type: PatchType,

    /// Target (what to patch)
    pub target: String,

    /// Patch data
    pub patch: String,

    /// Reasoning
    pub reasoning: String,

    /// Test requirements
    pub test_requirements: Vec<String>,
}

impl PatchProposal {
    /// Create a new patch proposal
    #[must_use]
    pub fn new(
        patch_type: PatchType,
        target: impl Into<String>,
        patch: impl Into<String>,
        reasoning: impl Into<String>,
    ) -> Self {
        Self {
            patch_type,
            target: target.into(),
            patch: patch.into(),
            reasoning: reasoning.into(),
            test_requirements: Vec::new(),
        }
    }
}

/// Types of patches
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PatchType {
    /// Prompt update
    Prompt,

    /// Policy update
    Policy,

    /// Routing heuristic update
    Routing,

    /// Configuration update
    Config,

    /// Tool addition/removal
    Tools,

    /// Other
    Other(String),
}

/// Deterministic state machine trait
///
/// Agents must implement this trait for deterministic behavior.
pub trait StateMachine: Send + Sync {
    /// Process input and produce output
    ///
    /// # Arguments
    /// - `state`: Current agent state
    /// - `observation`: Environment observation
    /// - `tool_responses`: Responses from previous tool calls
    /// - `context`: Deterministic execution context
    ///
    /// # Returns
    /// Next state and decision
    fn transition(
        &self,
        state: &AgentState,
        observation: &Observation,
        tool_responses: &[ToolResponse],
        context: &ExecutionContext,
    ) -> StateResult<Transition>;

    /// Get initial state
    fn initial_state(&self) -> AgentState;
}

/// Result type for state transitions
pub type StateResult<T> = core::result::Result<T, StateError>;

/// State machine errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StateError {
    /// Invalid state transition
    InvalidTransition { from: String, to: String },

    /// Missing required state
    MissingState(String),

    /// State corruption
    Corrupted(String),

    /// Invalid observation
    InvalidObservation(String),

    /// Transition failed
    TransitionFailed(String),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::InvalidTransition { from, to } => {
                write!(f, "Invalid transition: {} -> {}", from, to)
            }
            StateError::MissingState(s) => write!(f, "Missing state: {}", s),
            StateError::Corrupted(s) => write!(f, "State corrupted: {}", s),
            StateError::InvalidObservation(s) => write!(f, "Invalid observation: {}", s),
            StateError::TransitionFailed(s) => write!(f, "Transition failed: {}", s),
        }
    }
}

/// Transition output
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Transition {
    /// Next state
    pub state: AgentState,

    /// Decision
    pub decision: Decision,

    /// Transition hash
    pub transition_hash: Hash,
}

impl Transition {
    /// Create a new transition
    #[must_use]
    pub fn new(state: AgentState, decision: Decision) -> Self {
        let transition_hash = state.hash(); // Simplified for now
        Self {
            state,
            decision,
            transition_hash,
        }
    }
}

/// Execution context for state transitions
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionContext {
    /// Logical timestamp
    pub logical_time: u64,

    /// Run ID
    pub run_id: u64,

    /// Random seed (if applicable)
    pub seed: Option<u64>,
}

impl ExecutionContext {
    /// Create a new context
    #[must_use]
    pub fn new(logical_time: u64, run_id: u64) -> Self {
        Self {
            logical_time,
            run_id,
            seed: None,
        }
    }

    /// Create with seed
    #[must_use]
    pub fn with_seed(logical_time: u64, run_id: u64, seed: u64) -> Self {
        Self {
            logical_time,
            run_id,
            seed: Some(seed),
        }
    }
}

/// Tool response for state machine input
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolResponse {
    /// Tool name
    pub tool_name: String,

    /// Response data
    pub data: StateData,

    /// Success status
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,
}

impl ToolResponse {
    /// Create a successful response
    #[must_use]
    pub fn success(tool_name: impl Into<String>, data: StateData) -> Self {
        Self {
            tool_name: tool_name.into(),
            data,
            success: true,
            error: None,
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(tool_name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            data: StateData::Value(StateValue::None),
            success: false,
            error: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_initial() {
        let state = AgentState::initial();
        assert_eq!(state.version, 0);
        assert!(state.data.is_empty());
    }

    #[test]
    fn test_agent_state_set_get() {
        let mut state = AgentState::initial();
        state.set("test", StateData::Value(StateValue::U64(42)));

        assert!(state.get("test").is_some());
        assert_eq!(state.version, 1);
    }

    #[test]
    fn test_agent_state_rehash() {
        let mut state1 = AgentState::initial();
        state1.set("key", StateData::Value(StateValue::String("value".to_string())));

        let mut state2 = AgentState::initial();
        state2.set("key", StateData::Value(StateValue::String("value".to_string())));

        assert_eq!(state1.hash(), state2.hash());
    }

    #[test]
    fn test_observation() {
        let obs = Observation::new("test_observation", 100)
            .with_field("value", StateValue::U64(42));

        assert_eq!(obs.kind, "test_observation");
        assert_eq!(obs.logical_time, 100);
        assert_eq!(obs.data.len(), 1);
    }
}
