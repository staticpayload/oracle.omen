//! Storage for patches.

use crate::{
    patch::{Patch, PatchStatus},
    signature::{Signature, SignedPatch, SignerId},
};
use oracle_omen_core::hash::Hash;
use std::collections::BTreeMap;

/// Storage for patches and their status
#[derive(Clone, Default)]
pub struct PatchStore {
    patches: BTreeMap<String, (Patch, PatchStatus)>,
}

impl PatchStore {
    /// Create new patch store
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a patch
    pub fn add_patch(
        &mut self,
        id: String,
        patch: Patch,
        status: PatchStatus,
    ) -> Result<(), StoreError> {
        if self.patches.contains_key(&id) {
            return Err(StoreError::AlreadyExists(id));
        }
        self.patches.insert(id, (patch, status));
        Ok(())
    }

    /// Get a patch
    pub fn get_patch(&self, id: &str) -> Option<(Patch, PatchStatus)> {
        self.patches.get(id).cloned()
    }

    /// Update patch status
    pub fn update_status(&mut self, id: &str, status: PatchStatus) -> Result<(), StoreError> {
        if let Some((patch, _)) = self.patches.get_mut(id) {
            *patch = (patch.clone(), status);
            Ok(())
        } else {
            Err(StoreError::NotFound(id.to_string()))
        }
    }

    /// List all patches
    pub fn list_patches(&self) -> Vec<(String, Patch, PatchStatus)> {
        self.patches
            .iter()
            .map(|(id, (p, s))| (id.clone(), p.clone(), s.clone()))
            .collect()
    }

    /// Get patches by status
    pub fn get_by_status(&self, status: PatchStatus) -> Vec<Patch> {
        self.patches
            .values()
            .filter(|(_, s)| {
                // Simple comparison for variant, ignoring content
                matches!(s, PatchStatus::Proposed) if matches!(status, PatchStatus::Proposed)
                    || matches!(s, PatchStatus::Tested) if matches!(status, PatchStatus::Tested)
                    || matches!(s, PatchStatus::Audited) if matches!(status, PatchStatus::Audited)
                    || matches!(s, PatchStatus::Approved) if matches!(status, PatchStatus::Approved)
                    || matches!(s, PatchStatus::Applied) if matches!(status, PatchStatus::Applied)
                    || matches!(s, PatchStatus::Rejected { .. }) if matches!(status, PatchStatus::Rejected { .. })
                    || matches!(s, PatchStatus::RolledBack { .. }) if matches!(status, PatchStatus::RolledBack { .. })
            })
            .map(|(p, _)| p.clone())
            .collect()
    }
}

/// Storage for signed patches
#[derive(Clone, Default)]
pub struct SignedPatchStore {
    patches: BTreeMap<Hash, SignedPatch>,
    by_id: BTreeMap<String, Hash>,
}

impl SignedPatchStore {
    /// Create new signed patch store
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a signed patch
    pub fn add(&mut self, patch: SignedPatch) -> Result<(), StoreError> {
        let hash = patch.hash();
        if self.patches.contains_key(&hash) {
            return Err(StoreError::AlreadyExists(hash.to_string()));
        }
        self.by_id.insert(patch.patch.id.to_string(), hash);
        self.patches.insert(hash, patch);
        Ok(())
    }

    /// Get by hash
    pub fn get(&self, hash: &Hash) -> Option<SignedPatch> {
        self.patches.get(hash).cloned()
    }

    /// Get by ID
    pub fn get_by_id(&self, id: &str) -> Option<SignedPatch> {
        self.by_id.get(id).and_then(|h| self.patches.get(h).cloned())
    }

    /// List all
    pub fn list(&self) -> Vec<SignedPatch> {
        self.patches.values().cloned().collect()
    }
}

/// Store errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreError {
    AlreadyExists(String),
    NotFound(String),
    Corrupted(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::AlreadyExists(id) => write!(f, "Already exists: {}", id),
            StoreError::NotFound(id) => write!(f, "Not found: {}", id),
            StoreError::Corrupted(msg) => write!(f, "Corrupted: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch::{Patch, PatchId, PatchTarget, PatchType};

    #[test]
    fn test_patch_store() {
        let mut store = PatchStore::new();
        let patch = Patch::new(
            PatchId::new(1, 0),
            PatchType::Prompt,
            PatchTarget::SystemPrompt,
            "test".to_string(),
        );

        store
            .add_patch(patch.id.to_string(), patch.clone(), PatchStatus::Proposed)
            .unwrap();

        assert_eq!(store.patches.len(), 1);

        let (retrieved, status) = store.get_patch(&patch.id.to_string()).unwrap();
        assert_eq!(retrieved.id, patch.id);
        assert!(matches!(status, PatchStatus::Proposed));
    }

    #[test]
    fn test_patch_store_duplicate() {
        let mut store = PatchStore::new();
        let patch = Patch::new(
            PatchId::new(1, 0),
            PatchType::Prompt,
            PatchTarget::SystemPrompt,
            "test".to_string(),
        );

        store
            .add_patch(patch.id.to_string(), patch.clone(), PatchStatus::Proposed)
            .unwrap();

        let result = store.add_patch(patch.id.to_string(), patch, PatchStatus::Proposed);
        assert!(matches!(result, Err(StoreError::AlreadyExists(_))));
    }

    #[test]
    fn test_signed_patch_store() {
        let mut store = SignedPatchStore::new();
        let patch = Patch::new(
            PatchId::new(1, 0),
            PatchType::Prompt,
            PatchTarget::SystemPrompt,
            "test".to_string(),
        );

        let sig = Signature::from_bytes([1u8; 64]);
        let signer = SignerId::from_bytes([2u8; 32]);
        let signed = SignedPatch::new(patch, sig, signer);

        store.add(signed.clone()).unwrap();
        assert_eq!(store.list().len(), 1);

        let retrieved = store.get(&signed.hash()).unwrap();
        assert_eq!(retrieved.patch.id, signed.patch.id);
    }
}
