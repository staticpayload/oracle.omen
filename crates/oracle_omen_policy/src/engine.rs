//! Policy evaluation engine.
//!
//! Evaluates policies against execution context.

use crate::{
    schema::{CompiledCondition, CompiledPolicy, CompiledRule},
    lang::{Action, CompareOp, EvaluationResult, RuleKind, Value},
};
use std::collections::{BTreeMap, BTreeSet};

/// Execution context for policy evaluation
#[derive(Clone, Debug)]
pub struct EvalContext {
    /// Current capabilities
    pub capabilities: BTreeSet<String>,

    /// Tool being called
    pub tool: Option<String>,

    /// Memory key being accessed
    pub memory_key: Option<String>,

    /// Patch being proposed
    pub patch_type: Option<String>,

    /// Current state values
    pub state: BTreeMap<String, Value>,
}

impl EvalContext {
    /// Create empty context
    pub fn new() -> Self {
        Self {
            capabilities: BTreeSet::new(),
            tool: None,
            memory_key: None,
            patch_type: None,
            state: BTreeMap::new(),
        }
    }

    /// Check if has capability
    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.contains(cap)
            || self.capabilities.iter().any(|c| {
                // Check wildcard patterns
                c.ends_with('*') && cap.starts_with(&c[..c.len() - 1])
            })
    }
}

impl Default for EvalContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy engine
pub struct PolicyEngine {
    policies: Vec<CompiledPolicy>,
}

impl PolicyEngine {
    /// Create new policy engine
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    /// Add a policy
    pub fn add_policy(&mut self, policy: CompiledPolicy) {
        self.policies.push(policy);
    }

    /// Evaluate a tool call against policies
    pub fn evaluate_tool(&self, tool: &str, context: &EvalContext) -> EvaluationResult {
        let mut ctx = context.clone();
        ctx.tool = Some(tool.to_string());

        let mut results = Vec::new();

        for policy in &self.policies {
            for rule in &policy.rules {
                if rule.kind != RuleKind::Tool {
                    continue;
                }

                if self.evaluate_condition(&rule.condition, &ctx) {
                    results.push((policy.id.clone(), rule.clone()));
                }
            }
        }

        self.resolve_results(results, &format!("tool: {}", tool))
    }

    /// Evaluate a capability request
    pub fn evaluate_capability(&self, cap: &str, context: &EvalContext) -> EvaluationResult {
        let mut ctx = context.clone();

        // Add the requested capability temporarily for condition checking
        ctx.capabilities.insert(cap.to_string());

        let mut results = Vec::new();

        for policy in &self.policies {
            for rule in &policy.rules {
                if rule.kind != RuleKind::Capability {
                    continue;
                }

                // For capability rules, check if the rule mentions this capability
                let matches = if let CompiledCondition::HasCapability(rule_cap) = &rule.condition {
                    rule_cap == cap
                } else {
                    self.evaluate_condition(&rule.condition, &context)
                };

                if matches {
                    results.push((policy.id.clone(), rule.clone()));
                }
            }
        }

        self.resolve_results(results, &format!("capability: {}", cap))
    }

    /// Evaluate a patch proposal
    pub fn evaluate_patch(&self, patch_type: &str, context: &EvalContext) -> EvaluationResult {
        let mut ctx = context.clone();
        ctx.patch_type = Some(patch_type.to_string());

        let mut results = Vec::new();

        for policy in &self.policies {
            for rule in &policy.rules {
                if rule.kind != RuleKind::Patch {
                    continue;
                }

                if self.evaluate_condition(&rule.condition, &ctx) {
                    results.push((policy.id.clone(), rule.clone()));
                }
            }
        }

        self.resolve_results(results, &format!("patch: {}", patch_type))
    }

    /// Evaluate a condition
    fn evaluate_condition(&self, cond: &CompiledCondition, ctx: &EvalContext) -> bool {
        match cond {
            CompiledCondition::True => true,
            CompiledCondition::False => false,
            CompiledCondition::And(conds) => {
                conds.iter().all(|c| self.evaluate_condition(c, ctx))
            }
            CompiledCondition::Or(conds) => {
                conds.iter().any(|c| self.evaluate_condition(c, ctx))
            }
            CompiledCondition::Not(inner) => !self.evaluate_condition(inner, ctx),
            CompiledCondition::HasCapability(cap) => ctx.has_capability(cap),
            CompiledCondition::ToolEquals(tool) => ctx.tool.as_ref() == Some(tool),
            CompiledCondition::Compare { field, op, value } => {
                if let Some(state_val) = ctx.state.get(field) {
                    self.compare_values(state_val, op, value)
                } else {
                    false
                }
            }
            CompiledCondition::Custom(_) => false, // Custom conditions not supported
        }
    }

    /// Compare two values
    fn compare_values(&self, left: &Value, op: &CompareOp, right: &Value) -> bool {
        match (left, op, right) {
            (Value::String(a), CompareOp::Equal, Value::String(b)) => a == b,
            (Value::String(a), CompareOp::NotEqual, Value::String(b)) => a != b,
            (Value::Integer(a), CompareOp::Equal, Value::Integer(b)) => a == b,
            (Value::Integer(a), CompareOp::NotEqual, Value::Integer(b)) => a != b,
            (Value::Integer(a), CompareOp::Greater, Value::Integer(b)) => a > b,
            (Value::Integer(a), CompareOp::GreaterEqual, Value::Integer(b)) => a >= b,
            (Value::Integer(a), CompareOp::Less, Value::Integer(b)) => a < b,
            (Value::Integer(a), CompareOp::LessEqual, Value::Integer(b)) => a <= b,
            (Value::Boolean(a), CompareOp::Equal, Value::Boolean(b)) => a == b,
            _ => false,
        }
    }

    /// Resolve evaluation results
    fn resolve_results(
        &self,
        results: Vec<(crate::lang::PolicyId, CompiledRule)>,
        subject: &str,
    ) -> EvaluationResult {
        if results.is_empty() {
            // Default deny
            return EvaluationResult::denied(format!("No policy allows: {}", subject));
        }

        // Check for explicit denies
        for (_, rule) in &results {
            if matches!(rule.action, Action::Deny { .. }) {
                if let Action::Deny { reason } = &rule.action {
                    return EvaluationResult {
                        allowed: false,
                        action: rule.action.clone(),
                        matched_rules: vec![rule.name.clone()],
                        reason: reason.clone(),
                    };
                }
            }
        }

        // First allow wins
        for (policy_id, rule) in results {
            if matches!(rule.action, Action::Allow) {
                return EvaluationResult {
                    allowed: true,
                    action: rule.action.clone(),
                    matched_rules: vec![rule.name.clone()],
                    reason: format!("Allowed by policy {} rule {}", policy_id.name, rule.name),
                };
            }
        }

        EvaluationResult::denied(format!("No allow rule for: {}", subject))
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{Condition, Policy, PolicyId, Rule, RuleKind};
    use crate::compiler::PolicyCompiler;

    #[test]
    fn test_engine_allow_capability() {
        let mut policy = Policy::new("test", "1.0.0");
        policy.add_rule(Rule {
            name: "allow_fs_read".to_string(),
            kind: RuleKind::Capability,
            condition: Condition::HasCapability("fs:read".to_string()),
            action: Action::Allow,
        });

        let compiled = PolicyCompiler::compile(&policy).unwrap();
        let mut engine = PolicyEngine::new();
        engine.add_policy(compiled);

        let mut ctx = EvalContext::new();
        ctx.capabilities.insert("fs:read".to_string());

        let result = engine.evaluate_capability("fs:read", &ctx);
        assert!(result.allowed);
    }

    #[test]
    fn test_engine_deny_capability() {
        let mut policy = Policy::new("test", "1.0.0");
        policy.add_rule(Rule {
            name: "deny_fs_write".to_string(),
            kind: RuleKind::Capability,
            condition: Condition::HasCapability("fs:write".to_string()),
            action: Action::Deny {
                reason: "Write not allowed".to_string(),
            },
        });

        let compiled = PolicyCompiler::compile(&policy).unwrap();
        let mut engine = PolicyEngine::new();
        engine.add_policy(compiled);

        let mut ctx = EvalContext::new();
        ctx.capabilities.insert("fs:write".to_string());

        let result = engine.evaluate_capability("fs:write", &ctx);
        assert!(!result.allowed);
        assert_eq!(result.reason, "Write not allowed");
    }

    #[test]
    fn test_engine_default_deny() {
        let policy = Policy::new("test", "1.0.0");
        let compiled = PolicyCompiler::compile(&policy).unwrap();

        let mut engine = PolicyEngine::new();
        engine.add_policy(compiled);

        let ctx = EvalContext::new();
        let result = engine.evaluate_capability("fs:read", &ctx);
        assert!(!result.allowed);
    }

    #[test]
    fn test_engine_compare_condition() {
        use crate::lang::Condition;

        let mut policy = Policy::new("test", "1.0.0");
        policy.add_rule(Rule {
            name: "limit_iterations".to_string(),
            kind: RuleKind::Resource,
            condition: Condition::Compare {
                field: "iterations".to_string(),
                op: CompareOp::Less,
                value: Value::Integer(100),
            },
            action: Action::Allow,
        });

        let compiled = PolicyCompiler::compile(&policy).unwrap();
        let mut engine = PolicyEngine::new();
        engine.add_policy(compiled);

        let mut ctx = EvalContext::new();
        ctx.state.insert("iterations".to_string(), Value::Integer(50));

        let result = engine.evaluate_capability("any", &ctx);
        assert!(result.allowed);
    }
}
