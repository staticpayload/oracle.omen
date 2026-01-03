//! CRDT document store.

use crate::document::{Document, DocumentValue, MergeResult};
use oracle_omen_core::hash::Hash;
use std::collections::BTreeMap;

/// Memory store using CRDT documents
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemoryStore {
    /// Documents indexed by key
    documents: BTreeMap<String, Document>,

    /// Provenance index: event_id -> affected keys
    provenance: BTreeMap<u64, Vec<String>>,

    /// Store hash for verification
    store_hash: Hash,
}

impl MemoryStore {
    /// Create a new memory store
    pub fn new() -> Self {
        Self {
            documents: BTreeMap::new(),
            provenance: BTreeMap::new(),
            store_hash: Hash::zero(),
        }
    }

    /// Write a document
    pub fn write(&mut self, doc: Document) -> WriteResult {
        let key = doc.key.clone();
        let causal_event = doc.causal_event;

        // Record provenance
        self.provenance
            .entry(causal_event)
            .or_insert_with(Vec::new)
            .push(key.clone());

        // Insert or merge document
        let result = if let Some(existing) = self.documents.get_mut(&key) {
            existing.merge(&doc)
        } else {
            self.documents.insert(key.clone(), doc);
            MergeResult::Merged
        };

        self.rehash();
        WriteResult {
            key,
            result,
            store_hash: self.store_hash,
        }
    }

    /// Read a document by key
    pub fn read(&self, key: &str) -> Option<&Document> {
        self.documents.get(key)
    }

    /// Delete a document
    pub fn delete(&mut self, key: &str, causal_event: u64) -> DeleteResult {
        if self.documents.remove(key).is_some() {
            self.provenance
                .entry(causal_event)
                .or_insert_with(Vec::new)
            .push(format!("!{}", key)); // Prefix with ! to indicate deletion
            self.rehash();
            DeleteResult::Deleted
        } else {
            DeleteResult::NotFound
        }
    }

    /// Get all keys (deterministic order)
    pub fn keys(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }

    /// Get document count
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Get store hash
    pub fn hash(&self) -> Hash {
        self.store_hash
    }

    /// Get keys affected by an event
    pub fn keys_for_event(&self, event_id: u64) -> &[String] {
        self.provenance.get(&event_id).map_or(&[], |v| v)
    }

    /// Recompute store hash
    fn rehash(&mut self) {
        let hashes: Vec<Hash> = self.documents.values().map(|d| d.hash()).collect();
        self.store_hash = oracle_omen_core::hash::combine_hashes(&hashes);
    }

    /// Create a snapshot at current state
    pub fn snapshot(&self) -> StoreSnapshot {
        StoreSnapshot {
            document_hashes: self
                .documents
                .iter()
                .map(|(k, d)| (k.clone(), d.hash()))
                .collect(),
            store_hash: self.store_hash,
        }
    }

    /// Get state as of an event (temporal query)
    pub fn state_at_event(&self, _event_id: u64) -> MemoryStore {
        // TODO: Implement temporal queries using event log
        // For now, return current state
        self.clone()
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a write operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteResult {
    /// Key that was written
    pub key: String,

    /// Merge result
    pub result: MergeResult,

    /// New store hash
    pub store_hash: Hash,
}

/// Result of a delete operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeleteResult {
    /// Document was deleted
    Deleted,

    /// Document not found
    NotFound,
}

/// Store snapshot for replay/checkpointing
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoreSnapshot {
    /// Document hashes indexed by key
    pub document_hashes: BTreeMap<String, Hash>,

    /// Store hash
    pub store_hash: Hash,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_write_read() {
        let mut store = MemoryStore::new();
        let doc = Document::new("test", DocumentValue::String("value".to_string()), 1);

        store.write(doc);
        assert_eq!(store.len(), 1);

        let read = store.read("test");
        assert!(read.is_some());
        assert_eq!(read.unwrap().value, DocumentValue::String("value".to_string()));
    }

    #[test]
    fn test_memory_store_provenance() {
        let mut store = MemoryStore::new();
        store.write(Document::new("key1", DocumentValue::Integer(10), 1));
        store.write(Document::new("key2", DocumentValue::Integer(20), 1));

        let keys = store.keys_for_event(1);
        assert_eq!(keys, &["key1".to_string(), "key2".to_string()]);
    }

    #[test]
    fn test_memory_store_delete() {
        let mut store = MemoryStore::new();
        store.write(Document::new("test", DocumentValue::String("value".to_string()), 1));

        let result = store.delete("test", 2);
        assert_eq!(result, DeleteResult::Deleted);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_store_hash_changes() {
        let mut store = MemoryStore::new();
        let hash1 = store.hash();

        store.write(Document::new("test", DocumentValue::String("value".to_string()), 1));
        let hash2 = store.hash();

        assert_ne!(hash1, hash2);
        assert!(!hash2.is_zero());
    }
}
