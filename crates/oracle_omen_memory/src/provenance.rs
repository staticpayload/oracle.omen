//! Provenance tracking for memory operations.

use std::collections::BTreeMap;

/// Provenance record for a memory operation
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceRecord {
    /// Event that caused the operation
    pub causal_event: u64,

    /// Operation type
    pub operation: Operation,

    /// Key affected
    pub key: String,

    /// Value hash (if applicable)
    pub value_hash: Option<String>,

    /// Timestamp (logical)
    pub timestamp: u64,
}

impl ProvenanceRecord {
    /// Create a new provenance record
    pub fn new(causal_event: u64, operation: Operation, key: impl Into<String>) -> Self {
        Self {
            causal_event,
            operation,
            key: key.into(),
            value_hash: None,
            timestamp: 0,
        }
    }

    /// With value hash
    pub fn with_value_hash(mut self, hash: impl Into<String>) -> Self {
        self.value_hash = Some(hash.into());
        self
    }

    /// With timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Operation type
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Operation {
    /// Write operation
    Write,

    /// Delete operation
    Delete,

    /// Read operation
    Read,

    /// Merge operation
    Merge,
}

/// Provenance tracker
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceTracker {
    /// Records indexed by event ID
    records: BTreeMap<u64, Vec<ProvenanceRecord>>,

    /// Key history: key -> list of events that affected it
    key_history: BTreeMap<String, Vec<u64>>,
}

impl ProvenanceTracker {
    /// Create a new tracker
    pub fn new() -> Self {
        Self {
            records: BTreeMap::new(),
            key_history: BTreeMap::new(),
        }
    }

    /// Record a provenance event
    pub fn record(&mut self, record: ProvenanceRecord) {
        let event_id = record.causal_event;
        let key = record.key.clone();

        self.records.entry(event_id).or_default().push(record.clone());
        self.key_history.entry(key).or_default().push(event_id);
    }

    /// Get records for an event
    pub fn records_for_event(&self, event_id: u64) -> &[ProvenanceRecord] {
        self.records.get(&event_id).map_or(&[], |v| v)
    }

    /// Get history for a key
    pub fn history_for_key(&self, key: &str) -> &[u64] {
        self.key_history.get(key).map_or(&[], |v| v)
    }

    /// Trace why a value exists
    pub fn trace(&self, key: &str) -> Vec<&ProvenanceRecord> {
        let mut result = Vec::new();
        if let Some(events) = self.key_history.get(key) {
            for &event_id in events {
                if let Some(records) = self.records.get(&event_id) {
                    for record in records {
                        if record.key == key {
                            result.push(record);
                        }
                    }
                }
            }
        }
        result
    }

    /// Get all recorded events
    pub fn events(&self) -> Vec<u64> {
        self.records.keys().cloned().collect()
    }
}

impl Default for ProvenanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_record() {
        let record = ProvenanceRecord::new(1, Operation::Write, "test_key")
            .with_value_hash("abc123")
            .with_timestamp(100);

        assert_eq!(record.causal_event, 1);
        assert_eq!(record.key, "test_key");
        assert_eq!(record.value_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_provenance_tracker() {
        let mut tracker = ProvenanceTracker::new();

        tracker.record(ProvenanceRecord::new(1, Operation::Write, "key1"));
        tracker.record(ProvenanceRecord::new(1, Operation::Write, "key2"));
        tracker.record(ProvenanceRecord::new(2, Operation::Delete, "key1"));

        assert_eq!(tracker.records_for_event(1).len(), 2);
        assert_eq!(tracker.records_for_event(2).len(), 1);
        assert_eq!(tracker.history_for_key("key1"), &[1, 2]);
        assert_eq!(tracker.history_for_key("key2"), &[1]);
    }

    #[test]
    fn test_provenance_trace() {
        let mut tracker = ProvenanceTracker::new();

        tracker.record(ProvenanceRecord::new(1, Operation::Write, "test"));
        tracker.record(ProvenanceRecord::new(2, Operation::Write, "test"));
        tracker.record(ProvenanceRecord::new(3, Operation::Delete, "test"));

        let trace = tracker.trace("test");
        assert_eq!(trace.len(), 3);
    }
}
