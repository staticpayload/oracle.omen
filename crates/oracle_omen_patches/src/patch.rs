//! Patch types and definitions.

use crate::signature::{Signature, SignerId};
use oracle_omen_core::hash::Hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A patch proposal for self-modification
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    /// Patch ID (unique)
    pub id: PatchId,

    /// Patch type
    pub patch_type: PatchType,

    /// Target component
    pub target: PatchTarget,

    /// Patch data (serialized)
    pub data: BTreeMap<String, String>,

    /// Reasoning for the patch
    pub reasoning: String,

    /// Test requirements
    pub tests: Vec<TestRequirement>,

    /// Created at (logical time)
    pub created_at: u64,

    /// Created by (agent run ID)
    pub created_by: u64,
}

impl Patch {
    /// Create a new patch
    pub fn new(
        id: PatchId,
        patch_type: PatchType,
        target: PatchTarget,
        reasoning: String,
    ) -> Self {
        Self {
            id,
            patch_type,
            target,
            data: BTreeMap::new(),
            reasoning,
            tests: Vec::new(),
            created_at: 0,
            created_by: 0,
        }
    }

    /// Add a data field
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Add a test requirement
    pub fn with_test(mut self, test: TestRequirement) -> Self {
        self.tests.push(test);
        self
    }

    /// Compute patch hash
    pub fn hash(&self) -> Hash {
        Hash::from_canonical(self)
    }
}

/// Unique patch identifier
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PatchId {
    /// Agent run ID
    pub run_id: u64,

    /// Sequence within run
    pub sequence: u64,

    /// Checksum for uniqueness
    pub checksum: String,
}

impl PatchId {
    /// Create a new patch ID
    pub fn new(run_id: u64, sequence: u64) -> Self {
        Self {
            run_id,
            sequence,
            checksum: String::new(),
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.run_id, self.sequence)
    }
}

impl std::fmt::Display for PatchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.run_id, self.sequence)
    }
}

/// Type of patch
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchType {
    /// Update system prompt
    Prompt,

    /// Update policy
    Policy,

    /// Update routing heuristic
    Routing,

    /// Update configuration
    Config,

    /// Add or remove tool
    Tools,

    /// Update memory schema
    MemorySchema,

    /// Update planning parameters
    Planning,

    /// Custom patch type
    Custom(String),
}

/// Target of a patch
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchTarget {
    /// System prompt
    SystemPrompt,

    /// Named policy
    Policy(String),

    /// Named route
    Route(String),

    /// Config key
    Config(String),

    /// Tool by name
    Tool(String),

    /// Memory schema key
    MemorySchema(String),

    /// Custom target
    Custom(String),
}

/// Test requirement for a patch
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestRequirement {
    /// Test name
    pub name: String,

    /// Test type
    pub test_type: TestType,

    /// Expected outcome
    pub expected: TestOutcome,
}

/// Type of test
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestType {
    /// Unit test
    Unit,

    /// Integration test
    Integration,

    /// Property-based test
    Property,

    /// Determinism test
    Determinism,

    /// Replay test
    Replay,

    /// Custom test
    Custom(String),
}

/// Expected test outcome
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestOutcome {
    /// Must pass
    Pass,

    /// Must fail (for negative tests)
    Fail,

    /// May pass or fail
    Any,
}

/// Patch status in the lifecycle
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchStatus {
    /// Proposed, not yet reviewed
    Proposed,

    /// Passed test gate
    Tested,

    /// Passed audit gate
    Audited,

    /// Approved for application
    Approved,

    /// Applied to system
    Applied,

    /// Rejected
    Rejected { reason: String },

    /// Rolled back
    RolledBack { reason: String },
}

/// Signed patch with signature
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedPatch {
    /// The patch data
    pub patch: Patch,

    /// Signature
    pub signature: Signature,

    /// Signer identity
    pub signer: SignerId,
}

impl SignedPatch {
    /// Create a signed patch
    pub fn new(patch: Patch, signature: Signature, signer: SignerId) -> Self {
        Self {
            patch,
            signature,
            signer,
        }
    }

    /// Get patch hash
    pub fn hash(&self) -> Hash {
        self.patch.hash()
    }

    /// Verify signature
    pub fn verify(&self) -> bool {
        self.signature.verify(&self.patch.hash().to_hex().into_bytes(), &self.signer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_creation() {
        let id = PatchId::new(1, 0);
        let patch = Patch::new(
            id,
            PatchType::Prompt,
            PatchTarget::SystemPrompt,
            "Improve clarity".to_string(),
        );

        assert_eq!(patch.patch_type, PatchType::Prompt);
        assert_eq!(patch.target, PatchTarget::SystemPrompt);
    }

    #[test]
    fn test_patch_with_data() {
        let id = PatchId::new(1, 0);
        let patch = Patch::new(
            id,
            PatchType::Config,
            PatchTarget::Config("max_iterations".to_string()),
            "Increase limit".to_string(),
        )
        .with_data("value", "1000");

        assert_eq!(patch.data.get("value"), Some(&"1000".to_string()));
    }

    #[test]
    fn test_patch_stable_hash() {
        let id = PatchId::new(1, 0);
        let patch1 = Patch::new(
            id.clone(),
            PatchType::Prompt,
            PatchTarget::SystemPrompt,
            "Test".to_string(),
        )
        .with_data("key", "value");

        let patch2 = Patch::new(
            id,
            PatchType::Prompt,
            PatchTarget::SystemPrompt,
            "Test".to_string(),
        )
        .with_data("key", "value");

        // BTreeMap ensures stable ordering
        assert_eq!(patch1.hash(), patch2.hash());
    }
}
