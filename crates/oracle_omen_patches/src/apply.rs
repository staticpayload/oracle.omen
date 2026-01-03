//! Patch application and rollback.

use crate::{
    gate::GateResult,
    patch::{Patch, PatchId, PatchStatus, PatchTarget},
    signature::Signature,
    signature::SignerId,
    store::PatchStore,
};
use oracle_omen_core::{
    event::{Event, EventId, EventKind, EventPayload},
    hash::Hash,
    state::AgentState,
    time::LogicalTime,
};
use std::collections::BTreeMap;

/// Patch application error
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyError {
    /// Patch not found
    NotFound(String),

    /// Patch failed test gate
    TestFailed(String),

    /// Patch failed audit gate
    AuditFailed(String),

    /// Patch not approved
    NotApproved,

    /// Application failed
    ApplicationFailed(String),

    /// Rollback failed
    RollbackFailed(String),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::NotFound(id) => write!(f, "Patch not found: {}", id),
            ApplyError::TestFailed(reason) => write!(f, "Test failed: {}", reason),
            ApplyError::AuditFailed(reason) => write!(f, "Audit failed: {}", reason),
            ApplyError::NotApproved => write!(f, "Patch not approved"),
            ApplyError::ApplicationFailed(reason) => write!(f, "Application failed: {}", reason),
            ApplyError::RollbackFailed(reason) => write!(f, "Rollback failed: {}", reason),
        }
    }
}

impl std::error::Error for ApplyError {}

/// Patch application engine
pub struct PatchEngine {
    store: PatchStore,
    applied: BTreeMap<String, AppliedPatch>,
}

impl PatchEngine {
    /// Create new patch engine
    pub fn new(store: PatchStore) -> Self {
        Self {
            store,
            applied: BTreeMap::new(),
        }
    }

    /// Submit a patch proposal
    pub fn submit(&mut self, patch: Patch) -> Result<(), ApplyError> {
        let id = patch.id.to_string();
        let status = PatchStatus::Proposed;

        self.store.add_patch(id, patch, status).map_err(|e| {
            ApplyError::ApplicationFailed(format!("Store error: {}", e))
        })
    }

    /// Run test gate on a patch
    pub fn test_gate(
        &self,
        patch_id: &str,
        runner: &dyn crate::gate::TestRunner,
    ) -> Result<GateResult, ApplyError> {
        let (patch, _) = self.store.get_patch(patch_id)
            .ok_or_else(|| ApplyError::NotFound(patch_id.to_string()))?;

        Ok(crate::gate::TestGate::evaluate(&patch, runner))
    }

    /// Run audit gate on a patch
    pub fn audit_gate(
        &self,
        patch_id: &str,
        policy_ctx: &oracle_omen_policy::engine::EvalContext,
    ) -> Result<GateResult, ApplyError> {
        let (patch, _) = self.store.get_patch(patch_id)
            .ok_or_else(|| ApplyError::NotFound(patch_id.to_string()))?;

        Ok(crate::gate::AuditGate::evaluate(&patch, policy_ctx))
    }

    /// Approve a patch
    pub fn approve(
        &mut self,
        patch_id: &str,
        signature: Signature,
        signer: SignerId,
    ) -> Result<(), ApplyError> {
        let gate = crate::gate::ApprovalGate::new(vec![signer.clone()]);

        let result = gate.evaluate(&signature, &signer);
        if !result.is_passed() {
            return Err(ApplyError::NotApproved);
        }

        self.store.update_status(
            patch_id,
            PatchStatus::Approved,
        ).map_err(|e| ApplyError::ApplicationFailed(e.to_string()))
    }

    /// Apply a patch
    pub fn apply(
        &mut self,
        patch_id: &str,
        current_state: &mut AgentState,
    ) -> Result<ApplyResult, ApplyError> {
        let (patch, status) = self.store.get_patch(patch_id)
            .ok_or_else(|| ApplyError::NotFound(patch_id.to_string()))?;

        // Check patch is approved
        if !matches!(status, PatchStatus::Approved | PatchStatus::Tested) {
            return Err(ApplyError::NotApproved);
        }

        // Apply the patch
        let before_hash = current_state.hash();
        let result = self.apply_patch(&patch, current_state)?;
        let after_hash = current_state.hash();

        // Record application
        let rollback_data = result.rollback_data.clone();
        let applied = AppliedPatch {
            patch_id: patch_id.to_string(),
            patch_hash: patch.hash(),
            applied_at: LogicalTime::new(0, self.applied.len() as u64),
            before_hash,
            after_hash,
            rollback_data,
        };

        self.store.update_status(
            patch_id,
            PatchStatus::Applied,
        ).map_err(|e| ApplyError::ApplicationFailed(e.to_string()))?;

        self.applied.insert(patch_id.to_string(), applied);

        Ok(result)
    }

    /// Rollback a patch
    pub fn rollback(
        &mut self,
        patch_id: &str,
        current_state: &mut AgentState,
    ) -> Result<RollbackResult, ApplyError> {
        let applied = self.applied.get(patch_id)
            .ok_or_else(|| ApplyError::NotFound(patch_id.to_string()))?;

        // Restore previous state
        current_state.set(
            "_rollback",
            oracle_omen_core::state::StateData::Value(
                oracle_omen_core::state::StateValue::Hash(applied.before_hash),
            ),
        );

        self.store.update_status(
            patch_id,
            PatchStatus::RolledBack { reason: "Manual rollback".to_string() },
        ).map_err(|e| ApplyError::RollbackFailed(e.to_string()))?;

        Ok(RollbackResult {
            patch_id: patch_id.to_string(),
            restored_to: applied.before_hash,
        })
    }

    /// Apply a patch to state
    fn apply_patch(
        &self,
        patch: &Patch,
        state: &mut AgentState,
    ) -> Result<ApplyResult, ApplyError> {
        match &patch.target {
            PatchTarget::SystemPrompt => {
                // Update system prompt
                state.set(
                    "system_prompt",
                    oracle_omen_core::state::StateData::Value(
                        oracle_omen_core::state::StateValue::String(
                            patch.data.get("prompt").cloned().unwrap_or_default()
                        ),
                    ),
                );

                Ok(ApplyResult {
                    patch_id: patch.id.to_string(),
                    changes_made: vec!["system_prompt".to_string()],
                    rollback_data: BTreeMap::new(),
                })
            }
            PatchTarget::Config(key) => {
                if let Some(value) = patch.data.get("value") {
                    state.set(
                        format!("config.{}", key),
                        oracle_omen_core::state::StateData::Value(
                            oracle_omen_core::state::StateValue::String(value.clone())
                        ),
                    );
                }

                Ok(ApplyResult {
                    patch_id: patch.id.to_string(),
                    changes_made: vec![format!("config.{}", key)],
                    rollback_data: BTreeMap::new(),
                })
            }
            PatchTarget::Policy(name) => {
                // Update policy
                state.set(
                    format!("policy.{}", name),
                    oracle_omen_core::state::StateData::Value(
                        oracle_omen_core::state::StateValue::String(
                            patch.data.get("policy").cloned().unwrap_or_default()
                        ),
                    ),
                );

                Ok(ApplyResult {
                    patch_id: patch.id.to_string(),
                    changes_made: vec![format!("policy.{}", name)],
                    rollback_data: BTreeMap::new(),
                })
            }
            _ => Err(ApplyError::ApplicationFailed(
                "Target type not implemented".to_string()
            )),
        }
    }
}

/// Result of applying a patch
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplyResult {
    pub patch_id: String,
    pub changes_made: Vec<String>,
    pub rollback_data: BTreeMap<String, String>,
}

/// Result of rolling back a patch
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RollbackResult {
    pub patch_id: String,
    pub restored_to: Hash,
}

/// Record of an applied patch
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedPatch {
    pub patch_id: String,
    pub patch_hash: Hash,
    pub applied_at: LogicalTime,
    pub before_hash: Hash,
    pub after_hash: Hash,
    pub rollback_data: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch::PatchId;

    #[test]
    fn test_patch_apply_system_prompt() {
        let store = PatchStore::new();
        let mut engine = PatchEngine::new(store);
        let mut state = AgentState::initial();

        let patch = Patch::new(
            PatchId::new(1, 0),
            PatchType::Prompt,
            PatchTarget::SystemPrompt,
            "Update prompt".to_string(),
        )
        .with_data("prompt", "You are a helpful assistant.");

        engine.submit(patch.clone()).unwrap();
        engine.store.update_status(&patch.id.to_string(), PatchStatus::Approved).unwrap();

        let result = engine.apply(&patch.id.to_string(), &mut state).unwrap();
        assert!(result.changes_made.contains(&"system_prompt".to_string()));
    }

    #[test]
    fn test_patch_rollback() {
        let store = PatchStore::new();
        let mut engine = PatchEngine::new(store);
        let mut state = AgentState::initial();

        let patch = Patch::new(
            PatchId::new(1, 0),
            PatchType::Config,
            PatchTarget::Config("test".to_string()),
            "Test".to_string(),
        )
        .with_data("value", "42");

        engine.submit(patch.clone()).unwrap();
        engine.store.update_status(&patch.id.to_string(), PatchStatus::Approved).unwrap();
        engine.apply(&patch.id.to_string(), &mut state).unwrap();

        let result = engine.rollback(&patch.id.to_string(), &mut state);
        assert!(result.is_ok());
    }
}
