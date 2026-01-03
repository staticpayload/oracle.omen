//! Event log schema for deterministic replay.
//!
//! Every event must be:
//! - Serializable with stable ordering
//! - Contain all required fields
//! - Include hashes for verification
//! - Reference parent events for causal links

#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::fmt;

use crate::{
    capability::Capability,
    hash::Hash,
    serde_utils::StableMap,
    time::LogicalTime,
};

/// Unique event identifier
///
/// Combines run ID and sequence number for global uniqueness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct EventId {
    /// Run identifier
    pub run_id: u64,

    /// Sequence within run
    pub sequence: u64,
}

impl EventId {
    /// Create a new event ID
    #[must_use]
    pub const fn new(run_id: u64, sequence: u64) -> Self {
        Self { run_id, sequence }
    }

    /// Create initial event ID (sequence 0)
    #[must_use]
    pub const fn initial(run_id: u64) -> Self {
        Self {
            run_id,
            sequence: 0,
        }
    }

    /// Get next event ID
    #[must_use]
    pub const fn next(&self) -> Self {
        Self {
            run_id: self.run_id,
            sequence: self.sequence + 1,
        }
    }

    /// Convert to logical time
    #[must_use]
    pub const fn to_logical_time(&self) -> LogicalTime {
        LogicalTime {
            run_id: self.run_id,
            sequence: self.sequence,
        }
    }

    /// Get run ID
    #[must_use]
    pub const fn run_id(&self) -> u64 {
        self.run_id
    }

    /// Get sequence
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E({}:{})", self.run_id, self.sequence)
    }
}

impl From<LogicalTime> for EventId {
    fn from(t: LogicalTime) -> Self {
        Self {
            run_id: t.run_id,
            sequence: t.sequence,
        }
    }
}

/// Event kind/type
///
/// Each event kind has specific payload requirements.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventKind {
    /// Agent initialized
    AgentInit,

    /// State transition
    StateTransition,

    /// Tool requested
    ToolRequest,

    /// Tool response received
    ToolResponse,

    /// Capability denied
    CapabilityDenied,

    /// Observation received
    Observation,

    /// Decision made
    Decision,

    /// Memory write
    MemoryWrite,

    /// Memory read
    MemoryRead,

    /// Patch proposed
    PatchProposal,

    /// Patch applied
    PatchApplied,

    /// Patch rejected
    PatchRejected,

    /// Error occurred
    Error,

    /// Snapshot created
    Snapshot,

    /// Custom event kind
    Custom(String),
}

impl EventKind {
    /// Get event kind as string
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            EventKind::AgentInit => "agent_init",
            EventKind::StateTransition => "state_transition",
            EventKind::ToolRequest => "tool_request",
            EventKind::ToolResponse => "tool_response",
            EventKind::CapabilityDenied => "capability_denied",
            EventKind::Observation => "observation",
            EventKind::Decision => "decision",
            EventKind::MemoryWrite => "memory_write",
            EventKind::MemoryRead => "memory_read",
            EventKind::PatchProposal => "patch_proposal",
            EventKind::PatchApplied => "patch_applied",
            EventKind::PatchRejected => "patch_rejected",
            EventKind::Error => "error",
            EventKind::Snapshot => "snapshot",
            EventKind::Custom(s) => s,
        }
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Event - single entry in the event log
///
/// Events are append-only and immutable once written.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    /// Unique event identifier
    pub id: EventId,

    /// Parent event ID (for causal linkage)
    pub parent_id: Option<EventId>,

    /// Event kind
    pub kind: EventKind,

    /// Timestamp (logical, injected)
    pub timestamp: LogicalTime,

    /// Event payload
    pub payload: EventPayload,

    /// Hash of normalized payload
    pub payload_hash: Hash,

    /// State hash before event (if applicable)
    pub state_hash_before: Option<Hash>,

    /// State hash after event (if applicable)
    pub state_hash_after: Option<Hash>,
}

impl Event {
    /// Create a new event
    #[must_use]
    pub fn new(
        id: EventId,
        kind: EventKind,
        timestamp: LogicalTime,
        payload: EventPayload,
    ) -> Self {
        let payload_hash = payload.hash();
        Self {
            id,
            parent_id: None,
            kind,
            timestamp,
            payload,
            payload_hash,
            state_hash_before: None,
            state_hash_after: None,
        }
    }

    /// Create with parent ID
    #[must_use]
    pub fn with_parent(
        id: EventId,
        parent_id: EventId,
        kind: EventKind,
        timestamp: LogicalTime,
        payload: EventPayload,
    ) -> Self {
        let payload_hash = payload.hash();
        Self {
            id,
            parent_id: Some(parent_id),
            kind,
            timestamp,
            payload,
            payload_hash,
            state_hash_before: None,
            state_hash_after: None,
        }
    }

    /// Add state hashes
    #[must_use]
    pub fn with_state_hashes(mut self, before: Hash, after: Hash) -> Self {
        self.state_hash_before = Some(before);
        self.state_hash_after = Some(after);
        self
    }

    /// Compute event hash (full event hash)
    #[must_use]
    pub fn event_hash(&self) -> Hash {
        Hash::from_canonical(self)
    }

    /// Verify payload hash matches
    #[must_use]
    pub fn verify_payload_hash(&self) -> bool {
        self.payload_hash == self.payload.hash()
    }

    /// Check if this event follows another
    #[must_use]
    pub fn follows(&self, parent: &EventId) -> bool {
        self.parent_id.as_ref() == Some(parent)
    }
}

/// Event payload - typed data per event kind
///
/// Each payload is stable and serializable.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EventPayload {
    /// Agent initialization
    AgentInit(AgentInitPayload),

    /// State transition
    StateTransition(StateTransitionPayload),

    /// Tool request
    ToolRequest(ToolRequestPayload),

    /// Tool response
    ToolResponse(ToolResponsePayload),

    /// Capability denied
    CapabilityDenied(CapabilityDeniedPayload),

    /// Observation
    Observation(ObservationPayload),

    /// Decision
    Decision(DecisionPayload),

    /// Memory write
    MemoryWrite(MemoryPayload),

    /// Memory read
    MemoryRead(MemoryPayload),

    /// Patch proposal
    PatchProposal(PatchPayload),

    /// Patch applied
    PatchApplied(PatchPayload),

    /// Patch rejected
    PatchRejected(PatchRejectedPayload),

    /// Error
    Error(ErrorPayload),

    /// Snapshot
    Snapshot(SnapshotPayload),

    /// Raw JSON payload (for extensibility)
    Raw(StableMap<String, String>),
}

impl EventPayload {
    /// Get payload hash
    #[must_use]
    pub fn hash(&self) -> Hash {
        Hash::from_canonical(self)
    }

    /// Get event kind for this payload
    #[must_use]
    pub fn kind(&self) -> EventKind {
        match self {
            EventPayload::AgentInit(_) => EventKind::AgentInit,
            EventPayload::StateTransition(_) => EventKind::StateTransition,
            EventPayload::ToolRequest(_) => EventKind::ToolRequest,
            EventPayload::ToolResponse(_) => EventKind::ToolResponse,
            EventPayload::CapabilityDenied(_) => EventKind::CapabilityDenied,
            EventPayload::Observation(_) => EventKind::Observation,
            EventPayload::Decision(_) => EventKind::Decision,
            EventPayload::MemoryWrite(_) => EventKind::MemoryWrite,
            EventPayload::MemoryRead(_) => EventKind::MemoryRead,
            EventPayload::PatchProposal(_) => EventKind::PatchProposal,
            EventPayload::PatchApplied(_) => EventKind::PatchApplied,
            EventPayload::PatchRejected(_) => EventKind::PatchRejected,
            EventPayload::Error(_) => EventKind::Error,
            EventPayload::Snapshot(_) => EventKind::Snapshot,
            EventPayload::Raw(_) => EventKind::Custom("raw".to_string()),
        }
    }
}

/// Agent initialization payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentInitPayload {
    /// Agent type/name
    pub agent_type: String,

    /// Agent version
    pub agent_version: String,

    /// Initial configuration
    pub config: StableMap<String, String>,
}

/// State transition payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateTransitionPayload {
    /// Previous state hash
    pub from_hash: Hash,

    /// New state hash
    pub to_hash: Hash,

    /// Transition type
    pub transition_type: String,
}

/// Tool request payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolRequestPayload {
    /// Tool name
    pub tool_name: String,

    /// Tool version
    pub tool_version: String,

    /// Request hash
    pub request_hash: Hash,

    /// Required capabilities
    pub capabilities: Vec<Capability>,

    /// Input payload (JSON string for stability)
    pub input: String,
}

/// Tool response payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolResponsePayload {
    /// Tool name
    pub tool_name: String,

    /// Request hash being responded to
    pub request_hash: Hash,

    /// Response hash
    pub response_hash: Hash,

    /// Output payload (JSON string)
    pub output: String,

    /// Success status
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Capability denied payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CapabilityDeniedPayload {
    /// Requested capability
    pub capability: Capability,

    /// Tool that was denied
    pub tool_name: String,

    /// Reason for denial
    pub reason: String,
}

/// Observation payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObservationPayload {
    /// Observation type
    pub obs_type: String,

    /// Observation data (stable map)
    pub data: StableMap<String, String>,

    /// Source
    pub source: String,
}

/// Decision payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecisionPayload {
    /// Decision type
    pub decision_type: String,

    /// Decision data
    pub data: StableMap<String, String>,

    /// Reasoning
    pub reasoning: Option<String>,
}

/// Memory operation payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemoryPayload {
    /// Memory operation
    pub operation: String,

    /// Key
    pub key: String,

    /// Value hash (if write)
    pub value_hash: Option<Hash>,

    /// Causal event ID
    pub causal_event: EventId,
}

/// Patch payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PatchPayload {
    /// Patch type
    pub patch_type: String,

    /// Target
    pub target: String,

    /// Patch data hash
    pub patch_hash: Hash,

    /// Reasoning
    pub reasoning: String,
}

/// Patch rejected payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PatchRejectedPayload {
    /// Patch hash
    pub patch_hash: Hash,

    /// Rejection reason
    pub reason: String,

    /// Rejection stage
    pub stage: String,
}

/// Error payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErrorPayload {
    /// Error type
    pub error_type: String,

    /// Error message
    pub message: String,

    /// Component that failed
    pub component: String,

    /// Recoverable
    pub recoverable: bool,
}

/// Snapshot payload
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SnapshotPayload {
    /// Snapshot ID
    pub snapshot_id: String,

    /// Event sequence at snapshot
    pub at_sequence: u64,

    /// State hash at snapshot
    pub state_hash: Hash,

    /// Number of events prior to snapshot
    pub events_before: u64,
}

/// Event log - append-only sequence of events
///
/// The log is the source of truth for replay.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventLog {
    /// Run ID
    pub run_id: u64,

    /// Events in sequence
    events: Vec<Event>,

    /// Index by event ID for lookup
    index: BTreeMap<EventId, usize>,
}

impl EventLog {
    /// Create new event log
    #[must_use]
    pub fn new(run_id: u64) -> Self {
        Self {
            run_id,
            events: Vec::new(),
            index: BTreeMap::new(),
        }
    }

    /// Append an event
    ///
    /// Returns error if event ID doesn't match expected sequence.
    pub fn append(&mut self, event: Event) -> Result<(), EventLogError> {
        // Verify event belongs to this run
        if event.id.run_id != self.run_id {
            return Err(EventLogError::CorruptedLog(format!(
                "Event run_id mismatch: expected {}, got {}",
                self.run_id, event.id.run_id
            )));
        }

        // Verify sequence continuity
        let expected_seq = self.events.len() as u64;
        if event.id.sequence != expected_seq {
            return Err(EventLogError::CorruptedLog(format!(
                "Event sequence mismatch: expected {}, got {}",
                expected_seq, event.id.sequence
            )));
        }

        // Verify parent exists
        if let Some(parent) = event.parent_id {
            if !self.index.contains_key(&parent) {
                return Err(EventLogError::ParentNotFound(parent.to_string()));
            }
        }

        // Verify payload hash
        if !event.verify_payload_hash() {
            return Err(EventLogError::HashMismatch {
                expected: event.payload_hash.to_hex(),
                actual: event.payload.hash().to_hex(),
            });
        }

        let idx = self.events.len();
        self.index.insert(event.id, idx);
        self.events.push(event);
        Ok(())
    }

    /// Get event by ID
    #[must_use]
    pub fn get(&self, id: EventId) -> Option<&Event> {
        self.index.get(&id).and_then(|&idx| self.events.get(idx))
    }

    /// Get event by sequence
    #[must_use]
    pub fn get_by_sequence(&self, seq: u64) -> Option<&Event> {
        self.events.get(seq as usize)
    }

    /// Get last event
    #[must_use]
    pub fn last(&self) -> Option<&Event> {
        self.events.last()
    }

    /// Get all events
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Get event count
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Create snapshot at current position
    #[must_use]
    pub fn snapshot(&self) -> EventLogSnapshot {
        EventLogSnapshot {
            run_id: self.run_id,
            at_sequence: self.len() as u64,
            last_event_id: self.last().map(|e| e.id),
        }
    }
}

/// Event log snapshot for replay
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventLogSnapshot {
    pub run_id: u64,
    pub at_sequence: u64,
    pub last_event_id: Option<EventId>,
}

impl EventLogSnapshot {
    /// Create from log
    #[must_use]
    pub fn from_log(log: &EventLog) -> Self {
        log.snapshot()
    }
}

/// Event log errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventLogError {
    InvalidEventId(String),
    ParentNotFound(String),
    HashMismatch { expected: String, actual: String },
    CorruptedLog(String),
}

impl fmt::Display for EventLogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventLogError::InvalidEventId(id) => write!(f, "Invalid event ID: {}", id),
            EventLogError::ParentNotFound(id) => write!(f, "Parent not found: {}", id),
            EventLogError::HashMismatch { expected, actual } => {
                write!(f, "Hash mismatch: expected {}, got {}", expected, actual)
            }
            EventLogError::CorruptedLog(msg) => write!(f, "Corrupted log: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id() {
        let id = EventId::new(1, 0);
        assert_eq!(id.run_id(), 1);
        assert_eq!(id.sequence(), 0);
        assert_eq!(id.next().sequence(), 1);
    }

    #[test]
    fn test_event_log_append() {
        let mut log = EventLog::new(42);
        let event = Event::new(
            EventId::new(42, 0),
            EventKind::AgentInit,
            LogicalTime::initial(42),
            EventPayload::AgentInit(AgentInitPayload {
                agent_type: "test".to_string(),
                agent_version: "1.0.0".to_string(),
                config: StableMap::new(),
            }),
        );

        assert!(log.append(event).is_ok());
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_event_log_sequence_mismatch() {
        let mut log = EventLog::new(42);

        // Add first event
        let event1 = Event::new(
            EventId::new(42, 0),
            EventKind::AgentInit,
            LogicalTime::initial(42),
            EventPayload::AgentInit(AgentInitPayload {
                agent_type: "test".to_string(),
                agent_version: "1.0.0".to_string(),
                config: StableMap::new(),
            }),
        );
        log.append(event1).unwrap();

        // Try to add event with wrong sequence
        let event2 = Event::new(
            EventId::new(42, 2), // Should be 1
            EventKind::Observation,
            LogicalTime::new(42, 2),
            EventPayload::Observation(ObservationPayload {
                obs_type: "test".to_string(),
                data: StableMap::new(),
                source: "test".to_string(),
            }),
        );

        assert!(log.append(event2).is_err());
    }

    #[test]
    fn test_event_with_parent() {
        let mut log = EventLog::new(42);

        let parent = Event::new(
            EventId::new(42, 0),
            EventKind::AgentInit,
            LogicalTime::initial(42),
            EventPayload::AgentInit(AgentInitPayload {
                agent_type: "test".to_string(),
                agent_version: "1.0.0".to_string(),
                config: StableMap::new(),
            }),
        );
        log.append(parent).unwrap();

        let child = Event::with_parent(
            EventId::new(42, 1),
            EventId::new(42, 0),
            EventKind::Observation,
            LogicalTime::new(42, 1),
            EventPayload::Observation(ObservationPayload {
                obs_type: "test".to_string(),
                data: StableMap::new(),
                source: "test".to_string(),
            }),
        );

        assert!(log.append(child).is_ok());
        assert_eq!(log.len(), 2);
    }
}
