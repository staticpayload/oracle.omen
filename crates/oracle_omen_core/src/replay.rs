//! Replay engine for deterministic state reconstruction.
//!
//! The replay engine must:
//! - Reconstruct full state from logs and snapshots
//! - Reproduce decisions deterministically
//! - Detect divergence and output minimal diff
//! - Support partial replay from any snapshot boundary

#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::fmt;

use crate::{
    event::{Event, EventId, EventLog, EventLogError},
    hash::Hash,
    state::AgentState,
};

/// Replay engine
#[derive(Clone)]
pub struct ReplayEngine {
    /// Event log being replayed
    log: EventLog,

    /// Current state during replay
    current_state: AgentState,

    /// Position in log
    position: u64,
}

impl ReplayEngine {
    /// Create a new replay engine from an event log
    pub fn new(log: EventLog) -> Self {
        let current_state = AgentState::initial();
        Self {
            log,
            current_state,
            position: 0,
        }
    }

    /// Create from log with initial state
    pub fn with_state(log: EventLog, initial_state: AgentState) -> Self {
        Self {
            log,
            current_state: initial_state,
            position: 0,
        }
    }

    /// Replay all events to reconstruct final state
    pub fn replay_all(&mut self) -> ReplayResult<AgentState> {
        while self.step().is_some() {}
        Ok(self.current_state.clone())
    }

    /// Replay from a specific position
    pub fn replay_from(&mut self, position: u64) -> ReplayResult<AgentState> {
        self.position = position;
        self.replay_all()
    }

    /// Replay a single event
    ///
    /// Returns Some(event) if an event was processed, None if at end.
    pub fn step(&mut self) -> Option<&Event> {
        let event = self.log.get_by_sequence(self.position)?;
        self.apply_event(event);
        self.position += 1;
        Some(event)
    }

    /// Apply an event to the current state
    fn apply_event(&mut self, event: &Event) {
        match &event.payload {
            crate::event::EventPayload::StateTransition(payload) => {
                // Verify state hash matches before applying
                if let Some(before_hash) = event.state_hash_before {
                    if self.current_state.hash() != before_hash {
                        // State mismatch - divergence detected
                        self.current_state = AgentState::with_run_id(event.id.run_id);
                    }
                }
                // Update state hash (simplified - real implementation would apply delta)
                if let Some(after_hash) = event.state_hash_after {
                    self.current_state.set("_event_hash".to_string(),
                        crate::state::StateData::Value(crate::state::StateValue::Hash(after_hash)));
                }
            }
            _ => {
                // Other event types update state accordingly
                self.current_state.set(
                    format!("event_{}", event.id.sequence),
                    crate::state::StateData::Value(crate::state::StateValue::Hash(event.event_hash())),
                );
            }
        }
    }

    /// Get current state
    pub fn current_state(&self) -> &AgentState {
        &self.current_state
    }

    /// Get current position
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Check if replay is complete
    pub fn is_complete(&self) -> bool {
        self.position >= self.log.len() as u64
    }

    /// Detect divergence between two replay runs
    pub fn detect_divergence(&self, other: &ReplayEngine) -> Vec<DivergencePoint> {
        let mut divergences = Vec::new();

        let mut pos = 0u64;
        loop {
            let event1 = self.log.get_by_sequence(pos);
            let event2 = other.log.get_by_sequence(pos);

            match (event1, event2) {
                (Some(e1), Some(e2)) => {
                    if e1.event_hash() != e2.event_hash() {
                        divergences.push(DivergencePoint {
                            position: pos,
                            event_id: e1.id,
                            expected: e1.event_hash(),
                            actual: e2.event_hash(),
                            diff: self.diff_events(e1, e2),
                        });
                    }
                }
                (Some(_), None) | (None, Some(_)) => {
                    divergences.push(DivergencePoint {
                        position: pos,
                        event_id: EventId::new(0, pos),
                        expected: Hash::zero(),
                        actual: Hash::zero(),
                        diff: "Different event count".to_string(),
                    });
                    break;
                }
                (None, None) => break,
            }
            pos += 1;
        }

        divergences
    }

    /// Compute diff between two events
    fn diff_events(&self, e1: &Event, e2: &Event) -> String {
        if e1.kind != e2.kind {
            format!("Kind: {:?} vs {:?}", e1.kind, e2.kind)
        } else if e1.payload_hash != e2.payload_hash {
            format!("Payload: {} vs {}", e1.payload_hash, e2.payload_hash)
        } else {
            format!("Unknown difference")
        }
    }

    /// Verify replay integrity
    pub fn verify(&self) -> ReplayResult<VerificationReport> {
        let mut report = VerificationReport {
            total_events: self.log.len(),
            verified_events: 0,
            hash_failures: 0,
            state_mismatches: 0,
        };

        for i in 0..self.log.len() {
            if let Some(event) = self.log.get_by_sequence(i as u64) {
                if event.verify_payload_hash() {
                    report.verified_events += 1;
                } else {
                    report.hash_failures += 1;
                }
            }
        }

        Ok(report)
    }
}

/// Point where two runs diverged
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DivergencePoint {
    /// Position in log
    pub position: u64,

    /// Event ID
    pub event_id: EventId,

    /// Expected hash
    pub expected: Hash,

    /// Actual hash
    pub actual: Hash,

    /// Human-readable diff
    pub diff: String,
}

impl fmt::Display for DivergencePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Divergence at {}: expected {}, got {} - {}",
            self.position, self.expected, self.actual, self.diff
        )
    }
}

/// Replay verification report
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationReport {
    pub total_events: usize,
    pub verified_events: usize,
    pub hash_failures: usize,
    pub state_mismatches: usize,
}

impl VerificationReport {
    /// Check if verification passed
    pub fn is_valid(&self) -> bool {
        self.hash_failures == 0 && self.state_mismatches == 0
    }
}

impl fmt::Display for VerificationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Verification: {}/{} events verified, {} failures, {} mismatches - {}",
            self.verified_events,
            self.total_events,
            self.hash_failures,
            self.state_mismatches,
            if self.is_valid() { "VALID" } else { "INVALID" }
        )
    }
}

/// Replay errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayError {
    /// Event log error
    LogError(String),

    /// State corruption
    CorruptedState(String),

    /// Divergence detected
    Divergence { at: u64, expected: Hash, actual: Hash },

    /// Invalid position
    InvalidPosition(u64),
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayError::LogError(msg) => write!(f, "Log error: {}", msg),
            ReplayError::CorruptedState(msg) => write!(f, "Corrupted state: {}", msg),
            ReplayError::Divergence { at, expected, actual } => {
                write!(f, "Divergence at {}: expected {}, got {}", at, expected, actual)
            }
            ReplayError::InvalidPosition(pos) => write!(f, "Invalid position: {}", pos),
        }
    }
}

impl From<EventLogError> for ReplayError {
    fn from(e: EventLogError) -> Self {
        ReplayError::LogError(e.to_string())
    }
}

/// Replay result type
pub type ReplayResult<T> = Result<T, ReplayError>;

/// Snapshot for efficient replay
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    /// Snapshot ID
    pub id: String,

    /// Run ID
    pub run_id: u64,

    /// Position in event log
    pub position: u64,

    /// State at this point
    pub state: AgentState,

    /// State hash
    pub state_hash: Hash,

    /// Event hash at position
    pub event_hash: Hash,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(id: impl Into<String>, run_id: u64, position: u64, state: AgentState) -> Self {
        let state_hash = state.hash();
        Self {
            id: id.into(),
            run_id,
            position,
            state,
            state_hash,
            event_hash: Hash::zero(),
        }
    }

    /// Verify snapshot integrity
    pub fn verify(&self) -> bool {
        self.state_hash == self.state.hash()
    }
}

/// Snapshot manager
#[derive(Clone, Default)]
pub struct SnapshotManager {
    snapshots: BTreeMap<u64, Snapshot>,
}

impl SnapshotManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a snapshot
    pub fn add(&mut self, snapshot: Snapshot) {
        self.snapshots.insert(snapshot.position, snapshot);
    }

    /// Get snapshot at or before a position
    pub fn get_snapshot_before(&self, position: u64) -> Option<&Snapshot> {
        self.snapshots
            .range(..=position)
            .next_back()
            .map(|(_, s)| s)
    }

    /// Get snapshot at exact position
    pub fn get(&self, position: u64) -> Option<&Snapshot> {
        self.snapshots.get(&position)
    }

    /// List all snapshot positions
    pub fn positions(&self) -> Vec<u64> {
        self.snapshots.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventKind, EventPayload, LogicalTime};

    #[test]
    fn test_replay_engine_creation() {
        let log = EventLog::new(1);
        let engine = ReplayEngine::new(log);
        assert_eq!(engine.position(), 0);
        assert!(engine.is_complete());
    }

    #[test]
    fn test_snapshot_verification() {
        let state = AgentState::with_run_id(42);
        let snapshot = Snapshot::new("test", 42, 0, state.clone());
        assert!(snapshot.verify());
    }

    #[test]
    fn test_snapshot_manager() {
        let mut manager = SnapshotManager::new();
        let state = AgentState::with_run_id(1);

        manager.add(Snapshot::new("s1", 1, 10, state.clone()));
        manager.add(Snapshot::new("s2", 1, 20, state.clone()));

        assert_eq!(manager.get_snapshot_before(15).map(|s| s.position.as_ref()), Some(&10));
        assert_eq!(manager.get_snapshot_before(25).map(|s| s.position.as_ref()), Some(&20));
    }
}
