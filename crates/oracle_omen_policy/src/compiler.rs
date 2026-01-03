//! Policy compiler.
//!
//! Compiles policy documents into executable form.

use crate::{lang::Policy, schema::CompiledPolicy};
use std::collections::BTreeMap;

/// Policy compiler
pub struct PolicyCompiler;

impl PolicyCompiler {
    /// Compile a policy into executable form
    pub fn compile(policy: &Policy) -> Result<CompiledPolicy, CompileError> {
        let mut compiled = CompiledPolicy {
            id: policy.id(),
            rules: Vec::new(),
            metadata: policy.metadata.clone(),
        };

        for rule in &policy.rules {
            let compiled_rule = Self::compile_rule(rule)?;
            compiled.rules.push(compiled_rule);
        }

        // Validate compiled policy
        Self::validate(&compiled)?;

        Ok(compiled)
    }

    /// Compile a single rule
    fn compile_rule(rule: &crate::lang::Rule) -> Result<crate::schema::CompiledRule, CompileError> {
        Ok(crate::schema::CompiledRule {
            name: rule.name.clone(),
            kind: rule.kind.clone(),
            condition: Self::compile_condition(&rule.condition)?,
            action: rule.action.clone(),
        })
    }

    /// Compile a condition expression
    fn compile_condition(
        cond: &crate::lang::Condition,
    ) -> Result<crate::schema::CompiledCondition, CompileError> {
        match cond {
            crate::lang::Condition::True => Ok(crate::schema::CompiledCondition::True),
            crate::lang::Condition::False => Ok(crate::schema::CompiledCondition::False),
            crate::lang::Condition::And(conds) => {
                let compiled = conds
                    .iter()
                    .map(|c| Self::compile_condition(c))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(crate::schema::CompiledCondition::And(compiled))
            }
            crate::lang::Condition::Or(conds) => {
                let compiled = conds
                    .iter()
                    .map(|c| Self::compile_condition(c))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(crate::schema::CompiledCondition::Or(compiled))
            }
            crate::lang::Condition::Not(inner) => {
                let compiled = Self::compile_condition(inner)?;
                Ok(crate::schema::CompiledCondition::Not(Box::new(compiled)))
            }
            crate::lang::Condition::HasCapability(cap) => {
                Ok(crate::schema::CompiledCondition::HasCapability(cap.clone()))
            }
            crate::lang::Condition::ToolEquals(tool) => {
                Ok(crate::schema::CompiledCondition::ToolEquals(tool.clone()))
            }
            crate::lang::Condition::Compare { field, op, value } => {
                Ok(crate::schema::CompiledCondition::Compare {
                    field: field.clone(),
                    op: *op,
                    value: value.clone(),
                })
            }
            crate::lang::Condition::Custom(s) => {
                Ok(crate::schema::CompiledCondition::Custom(s.clone()))
            }
        }
    }

    /// Validate a compiled policy
    fn validate(policy: &CompiledPolicy) -> Result<(), CompileError> {
        // Check for rule name conflicts
        let mut names = std::collections::BTreeSet::new();
        for rule in &policy.rules {
            if !names.insert(&rule.name) {
                return Err(CompileError::DuplicateRule(rule.name.clone()));
            }
        }

        // Validate conditions are well-formed
        for rule in &policy.rules {
            Self::validate_condition(&rule.condition)?;
        }

        Ok(())
    }

    /// Validate a condition
    fn validate_condition(
        cond: &crate::schema::CompiledCondition,
    ) -> Result<(), CompileError> {
        match cond {
            crate::schema::CompiledCondition::And(conds) | crate::schema::CompiledCondition::Or(conds) => {
                for c in conds {
                    Self::validate_condition(c)?;
                }
            }
            crate::schema::CompiledCondition::Not(inner) => {
                Self::validate_condition(inner)?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Compilation errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    /// Duplicate rule name
    DuplicateRule(String),

    /// Invalid condition
    InvalidCondition(String),

    /// Invalid action
    InvalidAction(String),

    /// Circular dependency
    CircularDependency(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::DuplicateRule(name) => write!(f, "Duplicate rule: {}", name),
            CompileError::InvalidCondition(msg) => write!(f, "Invalid condition: {}", msg),
            CompileError::InvalidAction(msg) => write!(f, "Invalid action: {}", msg),
            CompileError::CircularDependency(msg) => write!(f, "Circular dependency: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{Action, Condition, Rule, RuleKind};

    #[test]
    fn test_compile_simple_policy() {
        let mut policy = Policy::new("test", "1.0.0");

        policy.add_rule(Rule {
            name: "allow_read".to_string(),
            kind: RuleKind::Capability,
            condition: Condition::HasCapability("fs:read".to_string()),
            action: Action::Allow,
        });

        let compiled = PolicyCompiler::compile(&policy).unwrap();
        assert_eq!(compiled.rules.len(), 1);
        assert_eq!(compiled.rules[0].name, "allow_read");
    }

    #[test]
    fn test_compile_duplicate_rule() {
        let mut policy = Policy::new("test", "1.0.0");

        let rule = Rule {
            name: "duplicate".to_string(),
            kind: RuleKind::Capability,
            condition: Condition::True,
            action: Action::Allow,
        };

        policy.add_rule(rule.clone());
        policy.add_rule(rule);

        let result = PolicyCompiler::compile(&policy);
        assert!(matches!(result, Err(CompileError::DuplicateRule(_))));
    }
}
