//! Compiled policy schema.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::lang::{Action, CompareOp, PolicyId, RuleKind, Value};

/// A compiled policy ready for evaluation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledPolicy {
    pub id: PolicyId,
    pub rules: Vec<CompiledRule>,
    pub metadata: BTreeMap<String, String>,
}

/// A compiled rule
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledRule {
    pub name: String,
    pub kind: RuleKind,
    pub condition: CompiledCondition,
    pub action: Action,
}

/// A compiled condition
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompiledCondition {
    True,
    False,
    And(Vec<CompiledCondition>),
    Or(Vec<CompiledCondition>),
    Not(Box<CompiledCondition>),
    HasCapability(String),
    ToolEquals(String),
    Compare {
        field: String,
        op: CompareOp,
        value: Value,
    },
    Custom(String),
}
