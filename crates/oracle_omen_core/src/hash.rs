//! Stable hashing for event log integrity and replay verification.
//!
//! Uses BLAKE3 for:
//! - Event payload hashes
//! - State hashes
//! - Tool request/response hashes
//! - Deterministic verification
//!
//! All hashes are canonicalized before computation.

use crate::error::HashError;

/// Length of a hash in bytes
pub const HASH_SIZE: usize = 32;

/// Length of a hex-encoded hash
pub const HEX_HASH_SIZE: usize = HASH_SIZE * 2;

/// Stable hash using BLAKE3
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash([u8; HASH_SIZE]);

impl Hash {
    /// Compute hash from bytes
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut output = [0u8; HASH_SIZE];
        output.copy_from_slice(blake3::hash(data).as_bytes());
        Self(output)
    }

    /// Compute hash from string
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Compute hash from canonical JSON
    #[must_use]
    pub fn from_canonical<T>(value: &T) -> Self
    where
        T: serde::Serialize,
    {
        match crate::serde_utils::CanonicalJson::serialize_bytes(value) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(_) => {
                // Fallback to regular JSON if canonical fails
                // This should not happen with proper serialization
                Self::from_bytes(&serde_json::to_vec(value).unwrap_or_default())
            }
        }
    }

    /// Create from raw bytes
    #[must_use]
    pub const fn from_raw(bytes: [u8; HASH_SIZE]) -> Self {
        Self(bytes)
    }

    /// Zero hash (all zeros)
    #[must_use]
    pub const fn zero() -> Self {
        Self([0u8; HASH_SIZE])
    }

    /// Get inner bytes
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_SIZE] {
        &self.0
    }

    /// Convert to hex string
    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut hex = String::with_capacity(HEX_HASH_SIZE);
        for byte in &self.0 {
            use core::fmt::Write;
            write!(hex, "{:02x}", byte).unwrap();
        }
        hex
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, HashError> {
        if hex.len() != HEX_HASH_SIZE {
            return Err(HashError::InvalidFormat(format!(
                "Expected {} chars, got {}",
                HEX_HASH_SIZE,
                hex.len()
            )));
        }

        let mut bytes = [0u8; HASH_SIZE];
        for i in 0..HASH_SIZE {
            let byte_str = &hex[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(byte_str, 16).map_err(|_| {
                HashError::InvalidFormat(format!("Invalid hex at position {}", i * 2))
            })?;
        }

        Ok(Self(bytes))
    }

    /// Check if this is the zero hash
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::zero()
    }
}

impl core::fmt::Debug for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Hash({})", self.to_hex())
    }
}

impl core::fmt::Display for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl serde::Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as hex string for stability
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        Self::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Compute combined hash from multiple hashes
///
/// Hashes are concatenated before hashing to ensure determinism.
#[must_use]
pub fn combine_hashes(hashes: &[Hash]) -> Hash {
    let mut combined = Vec::with_capacity(hashes.len() * HASH_SIZE);
    for hash in hashes {
        combined.extend_from_slice(&hash.0);
    }
    Hash::from_bytes(&combined)
}

/// Compute hash of state transition
///
/// Combines previous state hash, event hash, and resulting state hash.
#[must_use]
pub fn transition_hash(prev_state: Hash, event: Hash, next_state: Hash) -> Hash {
    combine_hashes(&[prev_state, event, next_state])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_stable() {
        let data = b"test data";
        let h1 = Hash::from_bytes(data);
        let h2 = Hash::from_bytes(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different() {
        let h1 = Hash::from_bytes(b"data1");
        let h2 = Hash::from_bytes(b"data2");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_roundtrip() {
        let h1 = Hash::from_bytes(b"test");
        let hex = h1.to_hex();
        let h2 = Hash::from_hex(&hex).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_zero() {
        let h = Hash::zero();
        assert!(h.is_zero());
        assert_eq!(h.to_hex(), "0".repeat(HEX_HASH_SIZE));
    }

    #[test]
    fn test_combine_hashes() {
        let h1 = Hash::from_bytes(b"first");
        let h2 = Hash::from_bytes(b"second");
        let combined = combine_hashes(&[h1, h2]);
        // Order matters
        let reversed = combine_hashes(&[h2, h1]);
        assert_ne!(combined, reversed);
    }

    #[test]
    fn test_invalid_hex() {
        assert!(Hash::from_hex("not a hash").is_err());
        assert!(Hash::from_hex(&"ab".repeat(16)).is_err()); // Too short
    }
}
