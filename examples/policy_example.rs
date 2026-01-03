// Example: Policy evaluation

use oracle_omen_core::capability::Capability;
use oracle_omen_policy::{
    compiler::PolicyCompiler,
    engine::{EvalContext, PolicyEngine},
    lang::{Action, Condition, Policy, PolicyId, Rule, RuleKind},
};

fn main() {
    println!("Oracle Omen - Policy Example");
    println!("==============================\n");

    // Create a policy
    let mut policy = Policy::new("example_policy", "1.0.0");

    // Allow file reads from /tmp
    policy.add_rule(Rule {
        name: "allow_tmp_read".to_string(),
        kind: RuleKind::Capability,
        condition: Condition::HasCapability("fs:read:/tmp".to_string()),
        action: Action::Allow,
    });

    // Deny all writes
    policy.add_rule(Rule {
        name: "deny_writes".to_string(),
        kind: RuleKind::Capability,
        condition: Condition::True,
        action: Action::Deny {
            reason: "Write operations not allowed".to_string(),
        },
    });

    // Compile policy
    let compiled = PolicyCompiler::compile(&policy).unwrap();
    println!("Policy compiled: {} with {} rules\n", compiled.id.name, compiled.rules.len());

    // Create policy engine
    let mut engine = PolicyEngine::new();
    engine.add_policy(compiled);

    // Test capability evaluation
    let mut ctx = EvalContext::new();
    ctx.capabilities.insert("fs:read:/tmp".to_string());

    let result = engine.evaluate_capability("fs:read:/tmp", &ctx);
    println!("fs:read:/tmp -> {} ({})", result.allowed, result.reason);

    let result = engine.evaluate_capability("fs:write:/tmp", &ctx);
    println!("fs:write:/tmp -> {} ({})", result.allowed, result.reason);
}
