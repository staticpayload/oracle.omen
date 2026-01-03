//! Policy language definition.
//!
//! Policies are declarative rules that govern agent behavior.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// A policy document
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    /// Policy name
    pub name: String,

    /// Policy version
    pub version: String,

    /// Policy rules
    pub rules: Vec<Rule>,

    /// Policy metadata
    pub metadata: BTreeMap<String, String>,
}

impl Policy {
    /// Create a new policy
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            rules: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Get policy ID
    pub fn id(&self) -> PolicyId {
        PolicyId {
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }
}

/// Unique policy identifier
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PolicyId {
    pub name: String,
    pub version: String,
}

/// A policy rule
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    /// Rule name
    pub name: String,

    /// Rule kind
    pub kind: RuleKind,

    /// Rule condition (when it applies)
    pub condition: Condition,

    /// Rule action (what to do)
    pub action: Action,
}

/// Rule kind
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleKind {
    /// Capability rule
    Capability,

    /// Tool rule
    Tool,

    /// Memory rule
    Memory,

    /// Patch rule
    Patch,

    /// Resource rule
    Resource,

    /// Custom rule
    Custom(String),
}

/// Condition expression
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// Always true
    True,

    /// Always false
    False,

    /// And of conditions
    And(Vec<Condition>),

    /// Or of conditions
    Or(Vec<Condition>),

    /// Not of condition
    Not(Box<Condition>),

    /// Capability check
    HasCapability(String),

    /// Tool check
    ToolEquals(String),

    /// Value comparison
    Compare {
        field: String,
        op: CompareOp,
        value: Value,
    },

    /// Custom condition
    Custom(String),
}

/// Comparison operator
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

/// Value in conditions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Integer(i64),
    Boolean(bool),
    List(Vec<Value>),
}

/// Rule action
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Allow the operation
    Allow,

    /// Deny the operation
    Deny { reason: String },

    /// Allow with modifications
    AllowModified {
        modifications: BTreeMap<String, String>,
    },

    /// Require additional approval
    RequireApproval {
        approver: String,
        reason: String,
    },

    /// Log and continue
    Log { level: LogLevel },

    /// Custom action
    Custom(String),
}

/// Log level
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Policy evaluation result
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvaluationResult {
    pub allowed: bool,
    pub action: Action,
    pub matched_rules: Vec<String>,
    pub reason: String,
}

impl EvaluationResult {
    /// Create an allow result
    pub fn allowed(reason: impl Into<String>) -> Self {
        Self {
            allowed: true,
            action: Action::Allow,
            matched_rules: Vec::new(),
            reason: reason.into(),
        }
    }

    /// Create a deny result
    pub fn denied(reason: impl Into<String>) -> Self {
        let reason_str = reason.into();
        Self {
            allowed: false,
            action: Action::Deny {
                reason: reason_str.clone(),
            },
            matched_rules: Vec::new(),
            reason: reason_str,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_creation() {
        let policy = Policy::new("test", "1.0.0");
        assert_eq!(policy.name, "test");
        assert_eq!(policy.version, "1.0.0");
    }

    #[test]
    fn test_policy_id() {
        let policy = Policy::new("test", "1.0.0");
        let id = policy.id();
        assert_eq!(id.name, "test");
        assert_eq!(id.version, "1.0.0");
    }

    #[test]
    fn test_condition_serialization() {
        let cond = Condition::And(vec![
            Condition::HasCapability("fs:read".to_string()),
            Condition::Compare {
                field: "path".to_string(),
                op: CompareOp::Equal,
                value: Value::String("/tmp".to_string()),
            },
        ]);

        let json = serde_json::to_string(&cond).unwrap();
        let _cond2: Condition = serde_json::from_str(&json).unwrap();
    }
}
