//! CRDT document types for memory store.

use oracle_omen_core::hash::Hash;
use std::collections::BTreeMap;

/// CRDT document
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    /// Document key
    pub key: String,

    /// Document value
    pub value: DocumentValue,

    /// Version vector for causality
    pub version: BTreeMap<String, u64>,

    /// Causal event ID
    pub causal_event: u64,

    /// Document hash
    pub hash: Hash,
}

impl Document {
    /// Create a new document
    pub fn new(key: impl Into<String>, value: DocumentValue, causal_event: u64) -> Self {
        let key = key.into();
        let mut doc = Self {
            key,
            value,
            version: BTreeMap::new(),
            causal_event,
            hash: Hash::zero(),
        };
        doc.rehash();
        doc
    }

    /// Recompute document hash
    fn rehash(&mut self) {
        self.hash = Hash::from_canonical(&(&self.key, &self.value, &self.version, self.causal_event));
    }

    /// Merge with another document (LWW-register semantics)
    pub fn merge(&mut self, other: &Document) -> MergeResult {
        if self.key != other.key {
            return MergeResult::KeyMismatch;
        }

        // Later causal event wins (LWW)
        if other.causal_event > self.causal_event {
            self.value = other.value.clone();
            self.causal_event = other.causal_event;
            self.rehash();
            return MergeResult::Merged;
        }

        MergeResult::Unchanged
    }

    /// Get document hash
    pub fn hash(&self) -> Hash {
        self.hash
    }
}

/// Result of a document merge
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeResult {
    /// Documents merged
    Merged,

    /// No changes made
    Unchanged,

    /// Key mismatch - cannot merge
    KeyMismatch,
}

/// Document value type
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DocumentValue {
    /// String value
    String(String),

    /// Bytes
    Bytes(Vec<u8>),

    /// Integer
    Integer(i64),

    /// Boolean
    Bool(bool),

    /// Map of nested documents
    Map(BTreeMap<String, DocumentValue>),

    /// List
    Vec(Vec<DocumentValue>),

    /// Null
    Null,

    /// Reference by hash
    Ref(Hash),
}

impl DocumentValue {
    /// Get type name
    pub fn type_name(&self) -> &str {
        match self {
            DocumentValue::String(_) => "string",
            DocumentValue::Bytes(_) => "bytes",
            DocumentValue::Integer(_) => "integer",
            DocumentValue::Bool(_) => "bool",
            DocumentValue::Map(_) => "map",
            DocumentValue::Vec(_) => "vec",
            DocumentValue::Null => "null",
            DocumentValue::Ref(_) => "ref",
        }
    }
}

impl From<String> for DocumentValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for DocumentValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for DocumentValue {
    fn from(n: i64) -> Self {
        Self::Integer(n)
    }
}

impl From<bool> for DocumentValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<Vec<u8>> for DocumentValue {
    fn from(v: Vec<u8>) -> Self {
        Self::Bytes(v)
    }
}

impl From<Hash> for DocumentValue {
    fn from(h: Hash) -> Self {
        Self::Ref(h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new("test", DocumentValue::String("value".to_string()), 1);
        assert_eq!(doc.key, "test");
        assert!(!doc.hash.is_zero());
    }

    #[test]
    fn test_document_merge() {
        let mut doc1 = Document::new("key", DocumentValue::Integer(10), 1);
        let doc2 = Document::new("key", DocumentValue::Integer(20), 2);

        let result = doc1.merge(&doc2);
        assert_eq!(result, MergeResult::Merged);
        assert_eq!(doc1.value, DocumentValue::Integer(20));
    }

    #[test]
    fn test_document_merge_lww() {
        let mut doc1 = Document::new("key", DocumentValue::Integer(20), 2);
        let doc2 = Document::new("key", DocumentValue::Integer(10), 1);

        let result = doc1.merge(&doc2);
        assert_eq!(result, MergeResult::Unchanged);
        assert_eq!(doc1.value, DocumentValue::Integer(20));
    }

    #[test]
    fn test_document_merge_key_mismatch() {
        let mut doc1 = Document::new("key1", DocumentValue::Integer(10), 1);
        let doc2 = Document::new("key2", DocumentValue::Integer(20), 2);

        let result = doc1.merge(&doc2);
        assert_eq!(result, MergeResult::KeyMismatch);
    }
}
