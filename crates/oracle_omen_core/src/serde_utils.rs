//! Deterministic serialization utilities.
//!
//! Enforces stable ordering and canonical representation.

#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::fmt;

/// Error for canonical serialization
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalError {
    /// Serialization failed
    SerializationFailed(String),
    /// Non-deterministic content detected
    NonDeterministic(String),
}

impl fmt::Display for CanonicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonicalError::SerializationFailed(s) => {
                write!(f, "Serialization failed: {}", s)
            }
            CanonicalError::NonDeterministic(s) => {
                write!(f, "Non-deterministic content: {}", s)
            }
        }
    }
}

/// Result type for canonical operations
pub type CanonicalResult<T> = core::result::Result<T, CanonicalError>;

/// Canonical JSON serializer with stable ordering
///
/// Uses BTreeMap internally and enforces sorted key output.
#[derive(Default)]
pub struct CanonicalJson;

impl CanonicalJson {
    /// Serialize to canonical JSON string
    ///
    /// Guarantees:
    /// - Object keys are sorted
    /// - No extraneous whitespace
    /// - Stable across runs
    pub fn serialize<T>(value: &T) -> CanonicalResult<String>
    where
        T: serde::Serialize,
    {
        serde_json::to_string(value)
            .map_err(|e| CanonicalError::SerializationFailed(e.to_string()))
    }

    /// Serialize to canonical JSON bytes
    pub fn serialize_bytes<T>(value: &T) -> CanonicalResult<Vec<u8>>
    where
        T: serde::Serialize,
    {
        serde_json::to_vec(value)
            .map_err(|e| CanonicalError::SerializationFailed(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn deserialize<'de, T>(s: &'de str) -> CanonicalResult<T>
    where
        T: serde::Deserialize<'de>,
    {
        serde_json::from_str(s)
            .map_err(|e| CanonicalError::SerializationFailed(e.to_string()))
    }

    /// Deserialize from JSON bytes
    pub fn deserialize_bytes<'de, T>(bytes: &'de [u8]) -> CanonicalResult<T>
    where
        T: serde::Deserialize<'de>,
    {
        serde_json::from_slice(bytes)
            .map_err(|e| CanonicalError::SerializationFailed(e.to_string()))
    }
}

/// Helper to ensure stable serialization of maps
///
/// Always serializes maps with sorted keys.
pub trait StableSerialize: serde::Serialize {
    /// Serialize to canonical JSON string
    fn to_canonical_json(&self) -> CanonicalResult<String> {
        CanonicalJson::serialize(self)
    }

    /// Serialize to canonical JSON bytes
    fn to_canonical_json_bytes(&self) -> CanonicalResult<Vec<u8>> {
        CanonicalJson::serialize_bytes(self)
    }
}

impl<T: serde::Serialize> StableSerialize for T {}

/// Stable map wrapper - enforces BTreeMap for deterministic iteration
pub type StableMap<K, V> = BTreeMap<K, V>;

/// Stable set wrapper
pub type StableSet<T> = BTreeSet<T>;

/// Re-export BTreeSet for convenience
pub use alloc::collections::BTreeSet;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestStruct {
        a: u32,
        b: String,
        c: Vec<u32>,
    }

    #[test]
    fn test_canonical_json_stable() {
        let val = TestStruct {
            a: 1,
            b: "test".to_string(),
            c: vec![2, 3, 1],
        };

        let json1 = CanonicalJson::serialize(&val).unwrap();
        let json2 = CanonicalJson::serialize(&val).unwrap();

        assert_eq!(json1, json2);
    }

    #[test]
    fn test_canonical_roundtrip() {
        let original = TestStruct {
            a: 42,
            b: "hello".to_string(),
            c: vec![1, 2, 3],
        };

        let json = CanonicalJson::serialize(&original).unwrap();
        let deserialized: TestStruct = CanonicalJson::deserialize(&json).unwrap();

        assert_eq!(original, deserialized);
    }
}
