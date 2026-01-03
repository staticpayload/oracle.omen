//! Plan to DAG compiler.
//!
//! Compiles planning DSL into validated DAG with deterministic execution order.

use crate::{dag::Dag, dsl::Plan, dag::DagNode, dag::DagNodeType};

/// Compiler for plans
pub struct PlanCompiler;

impl PlanCompiler {
    /// Compile a plan into a DAG
    pub fn compile(plan: &Plan) -> Result<Dag, CompileError> {
        let mut dag = Dag::new(plan.name.clone());

        // First pass: add all nodes
        for step in &plan.steps {
            let node_type = Self::step_to_node_type(step)?;
            let mut node = DagNode::new(step.id.clone(), node_type);
            node.capabilities = step.capabilities.clone();
            node.resources = step.resources.clone();
            node.failure_policy = step.failure_policy.clone();
            node.retry_policy = step.retry_policy.clone();
            node.timeout_policy = step.timeout_policy.clone();
            dag.add_node(node)?;
        }

        // Second pass: add edges (dependencies)
        for step in &plan.steps {
            for dep in &step.dependencies {
                dag.add_edge(dep.clone(), step.id.clone())?;
            }
        }

        // Validate the DAG
        dag.validate()?;

        Ok(dag)
    }

    /// Convert a plan step to a DAG node type
    fn step_to_node_type(step: &crate::dsl::PlanStep) -> Result<DagNodeType, CompileError> {
        match &step.step_type {
            crate::dsl::StepType::Tool { name, version, .. } => Ok(DagNodeType::Tool {
                name: name.clone(),
                version: version.clone(),
            }),
            crate::dsl::StepType::Observation { source, .. } => Ok(DagNodeType::Observation {
                source: source.clone(),
            }),
            crate::dsl::StepType::Decision { condition, .. } => Ok(DagNodeType::Decision {
                condition: condition.clone(),
            }),
            crate::dsl::StepType::Sequential { steps } => {
                // For sequential steps, we return a custom marker
                // The actual sequential execution is handled by the runtime
                Ok(DagNodeType::Custom {
                    type_name: "sequential".to_string(),
                    config: serde_json::json!({ "steps": steps }).into(),
                })
            }
            crate::dsl::StepType::Parallel { steps } => {
                Ok(DagNodeType::Custom {
                    type_name: "parallel".to_string(),
                    config: serde_json::json!({ "steps": steps }).into(),
                })
            }
            crate::dsl::StepType::Custom { type_name, config } => Ok(DagNodeType::Custom {
                type_name: type_name.clone(),
                config: config.clone(),
            }),
        }
    }
}

/// Compilation errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    /// Invalid step configuration
    InvalidStep { id: String, reason: String },

    /// Circular dependency detected
    CircularDependency { path: Vec<String> },

    /// Missing dependency
    MissingDependency { step: String, dep: String },

    /// DAG error
    Dag(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::InvalidStep { id, reason } => {
                write!(f, "Invalid step '{}': {}", id, reason)
            }
            CompileError::CircularDependency { path } => {
                write!(f, "Circular dependency detected: {:?}", path)
            }
            CompileError::MissingDependency { step, dep } => {
                write!(f, "Step '{}' depends on non-existent step '{}'", step, dep)
            }
            CompileError::Dag(msg) => write!(f, "DAG error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<crate::dag::DagError> for CompileError {
    fn from(e: crate::dag::DagError) -> Self {
        CompileError::Dag(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{Plan, PlanStep, StepType, ResourceAnnotation};

    #[test]
    fn test_compile_simple_plan() {
        let mut plan = Plan::new("test_plan");
        plan.add_step(PlanStep::new(
            "step1",
            StepType::Tool {
                name: "tool1".to_string(),
                version: "1.0.0".to_string(),
                input: serde_json::json!({}),
            },
        ));

        let dag = PlanCompiler::compile(&plan).unwrap();
        assert_eq!(dag.len(), 1);
        assert!(dag.node("step1").is_some());
    }

    #[test]
    fn test_compile_with_dependencies() {
        let mut plan = Plan::new("test_plan");
        plan.add_step(PlanStep::new("a", StepType::Tool {
            name: "tool1".to_string(),
            version: "1.0.0".to_string(),
            input: serde_json::json!({}),
        }));
        plan.add_step(
            PlanStep::new("b", StepType::Tool {
                name: "tool2".to_string(),
                version: "1.0.0".to_string(),
                input: serde_json::json!({}),
            })
            .depends_on("a"),
        );

        let dag = PlanCompiler::compile(&plan).unwrap();
        assert_eq!(dag.len(), 2);
        assert_eq!(dag.dependencies("b"), &["a".to_string()].into());
    }
}
