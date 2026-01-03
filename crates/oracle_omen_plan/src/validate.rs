//! Plan validation utilities.

use crate::{dag::Dag, dsl::Plan};

/// Validation result
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Plan validator
pub struct PlanValidator;

impl PlanValidator {
    /// Validate a plan before compilation
    pub fn validate(plan: &Plan) -> ValidationResult<()> {
        // Check for unique step IDs
        let mut seen = std::collections::HashSet::new();
        for step in &plan.steps {
            if !seen.insert(&step.id) {
                return Err(ValidationError::DuplicateStep(step.id.clone()));
            }
        }

        // Check all dependencies exist
        for step in &plan.steps {
            for dep in &step.dependencies {
                if !seen.contains(dep) {
                    return Err(ValidationError::DependencyNotFound {
                        step: step.id.clone(),
                        dependency: dep.clone(),
                    });
                }
            }
        }

        // Check resource bounds are sensible
        for step in &plan.steps {
            if step.resources.timeout_ms == 0 {
                return Err(ValidationError::InvalidResource {
                    step: step.id.clone(),
                    resource: "timeout_ms".to_string(),
                    reason: "must be positive".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate a DAG
    pub fn validate_dag(dag: &Dag) -> ValidationResult<()> {
        dag.validate().map_err(|e| ValidationError::DagError(e.to_string()))
    }
}

/// Validation errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    /// Duplicate step ID
    DuplicateStep(String),

    /// Dependency not found
    DependencyNotFound { step: String, dependency: String },

    /// Invalid resource specification
    InvalidResource { step: String, resource: String, reason: String },

    /// DAG error
    DagError(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateStep(id) => write!(f, "Duplicate step ID: {}", id),
            ValidationError::DependencyNotFound { step, dependency } => {
                write!(f, "Step '{}' depends on non-existent step '{}'", step, dependency)
            }
            ValidationError::InvalidResource { step, resource, reason } => {
                write!(f, "Invalid resource '{}' in step '{}': {}", resource, step, reason)
            }
            ValidationError::DagError(msg) => write!(f, "DAG error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}
