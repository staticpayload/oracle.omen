//! Capability enforcement for tool execution.

use oracle_omen_core::{capability::Capability, capability::CapabilitySet};

/// Capability checker
pub struct CapabilityChecker {
    /// Granted capabilities
    granted: CapabilitySet,
}

impl CapabilityChecker {
    /// Create a new checker with the given capabilities
    pub fn new(granted: CapabilitySet) -> Self {
        Self { granted }
    }

    /// Check if a capability is granted
    pub fn check(&self, capability: &Capability) -> CheckResult {
        if self.granted.has(capability) {
            CheckResult::Granted
        } else {
            CheckResult::Denied {
                capability: capability.clone(),
                reason: "Capability not granted".to_string(),
            }
        }
    }

    /// Check if all required capabilities are granted
    pub fn check_all(&self, required: &[Capability]) -> CheckResult {
        for cap in required {
            if !self.granted.has(cap) {
                return CheckResult::Denied {
                    capability: cap.clone(),
                    reason: "Required capability not granted".to_string(),
                };
            }
        }
        CheckResult::Granted
    }

    /// Get granted capabilities
    pub fn granted(&self) -> &CapabilitySet {
        &self.granted
    }
}

/// Result of a capability check
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckResult {
    /// Capability is granted
    Granted,

    /// Capability is denied
    Denied { capability: Capability, reason: String },
}

impl CheckResult {
    /// Check if granted
    pub fn is_granted(&self) -> bool {
        matches!(self, CheckResult::Granted)
    }

    /// Check if denied
    pub fn is_denied(&self) -> bool {
        matches!(self, CheckResult::Denied { .. })
    }

    /// Get denial reason if denied
    pub fn denial_reason(&self) -> Option<&str> {
        match self {
            CheckResult::Denied { reason, .. } => Some(reason),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_check() {
        let granted = CapabilitySet::new([
            Capability::new("fs:read:*"),
            Capability::new("network:http:get"),
        ]);

        let checker = CapabilityChecker::new(granted);

        assert!(checker.check(&Capability::new("fs:read:*")).is_granted());
        assert!(checker.check(&Capability::new("fs:write:*")).is_denied());
    }

    #[test]
    fn test_check_all() {
        let granted = CapabilitySet::new([
            Capability::new("fs:read:*"),
            Capability::new("network:http:get"),
        ]);

        let checker = CapabilityChecker::new(granted);

        assert!(checker
            .check_all(&[Capability::new("fs:read:*"), Capability::new("network:http:get")])
            .is_granted());

        assert!(checker
            .check_all(&[Capability::new("fs:read:*"), Capability::new("fs:write:*")])
            .is_denied());
    }
}
