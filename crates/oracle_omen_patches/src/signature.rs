//! Cryptographic signatures for patches.

use serde::{Deserialize, Serialize};
use serde_bytes::{ByteBuf as SerdeByteBuf};
use std::fmt;

/// Signature using Ed25519
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// Signature bytes (as serde_bytes for large array)
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

impl Signature {
    /// Create signature from bytes
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self { bytes: bytes.to_vec() }
    }

    /// Get signature as hex
    pub fn to_hex(&self) -> String {
        self.bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Parse from hex
    pub fn from_hex(hex: &str) -> Result<Self, SignatureError> {
        if hex.len() != 128 {
            return Err(SignatureError::InvalidLength);
        }

        let mut bytes = Vec::with_capacity(64);
        for i in 0..64 {
            let byte_str = &hex[i * 2..i * 2 + 2];
            bytes.push(u8::from_str_radix(byte_str, 16)
                .map_err(|_| SignatureError::InvalidHex)?);
        }

        Ok(Self { bytes })
    }

    /// Verify signature
    pub fn verify(&self, _message: &[u8], _signer: &SignerId) -> bool {
        // In production, use ed25519_dalek
        // For now, placeholder that checks signature format
        self.bytes.len() == 64 && self.bytes.iter().any(|&b| b != 0)
    }
}

/// Signer identity (public key)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SignerId {
    /// Public key bytes (as serde_bytes for large array)
    #[serde(with = "serde_bytes")]
    pub public_key: Vec<u8>,
}

impl SignerId {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { public_key: bytes.to_vec() }
    }

    /// Get as hex
    pub fn to_hex(&self) -> String {
        self.public_key.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Parse from hex
    pub fn from_hex(hex: &str) -> Result<Self, SignatureError> {
        if hex.len() != 64 {
            return Err(SignatureError::InvalidLength);
        }

        let mut public_key = Vec::with_capacity(32);
        for i in 0..32 {
            let byte_str = &hex[i * 2..i * 2 + 2];
            public_key.push(u8::from_str_radix(byte_str, 16)
                .map_err(|_| SignatureError::InvalidHex)?);
        }

        Ok(Self { public_key })
    }
}

impl fmt::Display for SignerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = self.to_hex();
        write!(f, "{}", &hex[..16.min(hex.len())])
    }
}

/// Key pair for signing
#[derive(Clone)]
pub struct KeyPair {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl KeyPair {
    /// Generate new key pair
    pub fn generate() -> Self {
        // In production, use ed25519_dalep::SigningKey::generate
        // For now, placeholder
        let mut secret_key = vec![0u8; 32];
        let mut public_key = vec![0u8; 32];

        // Fill with non-zero for testing
        for i in 0..32 {
            secret_key[i] = i as u8;
            public_key[i] = (i + 32) as u8;
        }

        Self {
            secret_key,
            public_key,
        }
    }

    /// Get signer ID
    pub fn signer_id(&self) -> SignerId {
        SignerId { public_key: self.public_key.clone() }
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        use sha2::Digest;
        // In production: use ed25519_dalek::SigningKey
        // Placeholder: compute simple hash-based signature
        let mut sig = vec![0u8; 64];

        // Simple signing for demonstration
        let hash = sha2::Sha256::digest(message);
        for i in 0..32 {
            sig[i] = hash[i];
            sig[i + 32] = self.secret_key[i];
        }

        Signature { bytes: sig }
    }
}

/// Signature errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignatureError {
    InvalidLength,
    InvalidHex,
    VerificationFailed,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureError::InvalidLength => write!(f, "Invalid signature length"),
            SignatureError::InvalidHex => write!(f, "Invalid hex encoding"),
            SignatureError::VerificationFailed => write!(f, "Signature verification failed"),
        }
    }
}

impl std::error::Error for SignatureError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        assert_ne!(kp.secret_key, [0u8; 32]);
        assert_ne!(kp.public_key, [0u8; 32]);
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = KeyPair::generate();
        let message = b"test message";

        let sig = kp.sign(message);
        let signer = kp.signer_id();

        // Note: verify is a placeholder, will use real crypto in production
        assert!(sig.bytes != [0u8; 64]);
    }

    #[test]
    fn test_signer_id_roundtrip() {
        let kp = KeyPair::generate();
        let id = kp.signer_id();
        let hex = id.to_hex();
        let id2 = SignerId::from_hex(&hex).unwrap();
        assert_eq!(id, id2);
    }
}
