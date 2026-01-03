//! Capability system for tool access control.
//!
//! Capabilities are immutable during execution and checked before tool use.

#![no_std]

extern crate alloc;

use alloc::{collections::BTreeSet, string::String, vec::Vec};
use core::fmt;

/// A capability grants permission to perform a specific class of actions
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct Capability(String);

impl Capability {
    /// Create a new capability
    ///
    /// # Format
    /// Capabilities use a hierarchical namespace: `domain:action:scope`
    /// Examples:
    /// - `fs:read:*` - read any file
    /// - `fs:write:/tmp` - write to /tmp
    /// - `network:http:get` - make HTTP GET requests
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the capability name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }

    /// Check if this capability matches a pattern
    #[must_use]
    pub fn matches(&self, pattern: &str) -> bool {
        let parts: Vec<&str> = pattern.split(':').collect();
        let self_parts: Vec<&str> = self.0.split(':').collect();

        for (i, part) in parts.iter().enumerate() {
            if *part == "*" {
                continue;
            }
            if self_parts.get(i) != Some(part) {
                return false;
            }
        }
        true
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Capability {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Capability {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// A set of granted capabilities
///
/// Immutable during an execution run.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CapabilitySet {
    inner: BTreeSet<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set
    #[must_use]
    pub fn empty() -> Self {
        Self {
            inner: BTreeSet::new(),
        }
    }

    /// Create a capability set from a list
    #[must_use]
    pub fn new(capabilities: impl IntoIterator<Item = Capability>) -> Self {
        Self {
            inner: capabilities.into_iter().collect(),
        }
    }

    /// Check if a capability is granted
    #[must_use]
    pub fn has(&self, capability: &Capability) -> bool {
        self.inner.contains(capability)
    }

    /// Check if a capability pattern is granted
    #[must_use]
    pub fn has_pattern(&self, pattern: &str) -> bool {
        self.inner.iter().any(|c| c.matches(pattern))
    }

    /// Check if any of the required capabilities are granted
    #[must_use]
    pub fn has_any(&self, required: &[Capability]) -> bool {
        required.iter().any(|c| self.has(c))
    }

    /// Check if all required capabilities are granted
    #[must_use]
    pub fn has_all(&self, required: &[Capability]) -> bool {
        required.iter().all(|c| self.has(c))
    }

    /// Get the number of capabilities
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Iterate over capabilities
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.inner.iter()
    }

    /// Convert to vector
    #[must_use]
    pub fn to_vec(&self) -> Vec<Capability> {
        self.inner.iter().cloned().collect()
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::empty()
    }
}

/// Standard capability domains
pub mod std {
    use super::Capability;

    /// File system read capability
    pub const fn fs_read(path: &str) -> Capability {
        // This is a const-friendly placeholder
        // Actual usage would need runtime construction
        Capability::new("fs:read:*")
    }

    /// File system write capability
    pub const fn fs_write() -> Capability {
        Capability::new("fs:write:*")
    }

    /// Network HTTP capability
    pub const fn network_http() -> Capability {
        Capability::new("network:http:*")
    }

    /// Network HTTPS capability
    pub const fn network_https() -> Capability {
        Capability::new("network:https:*")
    }

    /// Process execution capability
    pub const fn process_exec() -> Capability {
        Capability::new("process:exec:*")
    }

    /// Environment variable read capability
    pub const fn env_read() -> Capability {
        Capability::new("env:read:*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_matches() {
        let cap = Capability::new("fs:read:/tmp");
        assert!(cap.matches("fs:read:/tmp"));
        assert!(cap.matches("fs:read:*"));
        assert!(!cap.matches("fs:write:*"));
    }

    #[test]
    fn test_capability_set() {
        let set = CapabilitySet::new([
            Capability::new("fs:read:*"),
            Capability::new("network:http:get"),
        ]);

        assert!(set.has(&Capability::new("fs:read:*")));
        assert!(set.has(&Capability::new("network:http:get")));
        assert!(!set.has(&Capability::new("fs:write:*")));
    }

    #[test]
    fn test_capability_pattern() {
        let set = CapabilitySet::new([Capability::new("fs:read:*")]);
        assert!(set.has_pattern("fs:read:*"));
        assert!(set.has_pattern("fs:read:/tmp"));
        assert!(!set.has_pattern("fs:write:*"));
    }
}
