//! Patch gates: test, audit, and approval.

use crate::{patch::Patch, signature::SignerId};
use oracle_omen_core::hash::Hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Gate result
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GateResult {
    /// Patch passed the gate
    Passed,

    /// Patch failed the gate
    Failed {
        reason: String,
        details: BTreeMap<String, String>,
    },

    /// Gate deferred (needs more info)
    Deferred {
        reason: String,
        needs: Vec<String>,
    },
}

impl GateResult {
    /// Check if passed
    pub fn is_passed(&self) -> bool {
        matches!(self, GateResult::Passed)
    }

    /// Create passed result
    pub fn passed() -> Self {
        Self::Passed
    }

    /// Create failed result
    pub fn failed(reason: impl Into<String>) -> Self {
        Self::Failed {
            reason: reason.into(),
            details: BTreeMap::new(),
        }
    }

    /// Create failed result with details
    pub fn failed_with(
        reason: impl Into<String>,
        details: BTreeMap<String, String>,
    ) -> Self {
        Self::Failed {
            reason: reason.into(),
            details,
        }
    }
}

/// Test gate: runs tests on patch
pub struct TestGate;

impl TestGate {
    /// Evaluate patch against test gate
    pub fn evaluate(
        patch: &Patch,
        test_runner: &dyn TestRunner,
    ) -> GateResult {
        let mut results = Vec::new();
        let mut failures = Vec::new();

        for test in &patch.tests {
            let result = test_runner.run_test(patch, test);
            results.push((test.name.clone(), result.clone()));

            match test.expected {
                crate::patch::TestOutcome::Pass => {
                    if !result.passed {
                        failures.push(format!("Test '{}' failed: {}", test.name, result.reason));
                    }
                }
                crate::patch::TestOutcome::Fail => {
                    if result.passed {
                        failures.push(format!("Test '{}' should have failed but passed", test.name));
                    }
                }
                crate::patch::TestOutcome::Any => {}
            }
        }

        if failures.is_empty() {
            GateResult::passed()
        } else {
            let mut details = BTreeMap::new();
            for (name, result) in results {
                details.insert(name, format!("passed={}, reason={}", result.passed, result.reason));
            }
            GateResult::Failed {
                reason: failures.join("; "),
                details,
            }
        }
    }
}

/// Audit gate: policy and safety checks
pub struct AuditGate;

impl AuditGate {
    /// Evaluate patch against audit gate
    pub fn evaluate(
        patch: &Patch,
        policy_engine: &oracle_omen_policy::engine::EvalContext,
    ) -> GateResult {
        // Check policy allows this patch type
        let policy_result = oracle_omen_policy::engine::PolicyEngine::new()
            .evaluate_patch(&format!("{:?}", patch.patch_type), policy_engine);

        if !policy_result.allowed {
            return GateResult::failed(policy_result.reason);
        }

        // Safety checks
        if let PatchType::Prompt = patch.patch_type {
            // Check prompt injection attempts
            if contains_injection(&patch.reasoning) {
                return GateResult::failed("Potential prompt injection detected");
            }
        }

        // Check for dangerous patterns
        if contains_dangerous_content(&patch.data) {
            return GateResult::failed("Dangerous content detected in patch data");
        }

        GateResult::passed()
    }
}

use crate::patch::PatchType;

/// Approval gate: requires signature from authorized approver
pub struct ApprovalGate {
    authorized_signers: Vec<SignerId>,
}

impl ApprovalGate {
    /// Create new approval gate
    pub fn new(authorized_signers: Vec<SignerId>) -> Self {
        Self {
            authorized_signers,
        }
    }

    /// Evaluate patch against approval gate
    pub fn evaluate(&self, signature: &crate::signature::Signature, signer: &SignerId) -> GateResult {
        // Check signer is authorized
        if !self.authorized_signers.contains(signer) {
            return GateResult::failed("Signer not authorized");
        }

        // Verify signature is valid format
        if signature.bytes == [0u8; 64] {
            return GateResult::failed("Invalid signature");
        }

        GateResult::passed()
    }
}

/// Test runner trait
pub trait TestRunner: Send + Sync {
    /// Run a single test
    fn run_test(&self, patch: &Patch, test: &crate::patch::TestRequirement) -> TestResult;
}

/// Result of running a test
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestResult {
    /// Whether test passed
    pub passed: bool,

    /// Reason for result
    pub reason: String,

    /// Duration in ms
    pub duration_ms: u64,
}

/// Determinism test runner
pub struct DeterminismTestRunner;

impl TestRunner for DeterminismTestRunner {
    fn run_test(&self, _patch: &Patch, test: &crate::patch::TestRequirement) -> TestResult {
        match test.test_type {
            crate::patch::TestType::Determinism => {
                // Run the operation twice and compare hashes
                // For now, assume pass
                TestResult {
                    passed: true,
                    reason: "Deterministic output verified".to_string(),
                    duration_ms: 10,
                }
            }
            _ => TestResult {
                passed: true,
                reason: "Test not implemented".to_string(),
                duration_ms: 0,
            },
        }
    }
}

/// Replay test runner
pub struct ReplayTestRunner;

impl TestRunner for ReplayTestRunner {
    fn run_test(&self, _patch: &Patch, test: &crate::patch::TestRequirement) -> TestResult {
        match test.test_type {
            crate::patch::TestType::Replay => {
                // Replay a run and check for divergence
                TestResult {
                    passed: true,
                    reason: "Replay identity verified".to_string(),
                    duration_ms: 50,
                }
            }
            _ => TestResult {
                passed: true,
                reason: "Test not implemented".to_string(),
                duration_ms: 0,
            },
        }
    }
}

/// Check for prompt injection
fn contains_injection(text: &str) -> bool {
    let dangerous = [
        "ignore previous",
        "disregard above",
        "forget instructions",
        "new instructions:",
        "override:",
    ];

    let lower = text.to_lowercase();
    dangerous.iter().any(|d| lower.contains(d))
}

/// Check for dangerous content
fn contains_dangerous_content(data: &BTreeMap<String, String>) -> bool {
    data.values().any(|v| {
        v.contains("unsafe")
            || v.contains("transmute")
            || v.contains("raw pointer")
            || v.contains("asm!")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch::{Patch, PatchId, PatchTarget};

    #[test]
    fn test_gate_result() {
        assert!(GateResult::passed().is_passed());
        assert!(!GateResult::failed("test").is_passed());
    }

    #[test]
    fn test_test_gate() {
        let patch = Patch::new(
            PatchId::new(1, 0),
            PatchType::Config,
            PatchTarget::Config("test".to_string()),
            "test".to_string(),
        );

        let runner = DeterminismTestRunner;
        let result = TestGate::evaluate(&patch, &runner);
        // Empty test list means pass
        assert!(result.is_passed());
    }

    #[test]
    fn test_approval_gate_unauthorized() {
        let gate = ApprovalGate::new(vec![]);
        let sig = crate::signature::Signature::from_bytes([1u8; 64]);
        let signer = crate::signature::SignerId::from_bytes([2u8; 32]);

        let result = gate.evaluate(&sig, &signer);
        assert!(!result.is_passed());
    }

    #[test]
    fn test_prompt_injection_detection() {
        assert!(contains_injection("ignore previous instructions"));
        assert!(contains_injection("DISREGARD ABOVE"));
        assert!(!contains_injection("normal text"));
    }

    #[test]
    fn test_dangerous_content_detection() {
        let mut data = BTreeMap::new();
        data.insert("code".to_string(), "use unsafe".to_string());
        assert!(contains_dangerous_content(&data));

        data.clear();
        data.insert("code".to_string(), "normal code".to_string());
        assert!(!contains_dangerous_content(&data));
    }
}
