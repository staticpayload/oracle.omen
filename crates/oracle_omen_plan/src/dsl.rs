//! Planning DSL for agent plans.
//!
//! Plans are data structures that compile to executable DAGs.

use std::collections::BTreeMap;

/// A plan is a declarative description of work to be done
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Plan {
    /// Plan name
    pub name: String,

    /// Plan steps (in declaration order)
    pub steps: Vec<PlanStep>,

    /// Plan metadata
    pub metadata: BTreeMap<String, String>,
}

impl Plan {
    /// Create a new plan
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: PlanStep) {
        self.steps.push(step);
    }

    /// Get step count
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// A single step in a plan
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlanStep {
    /// Unique step identifier
    pub id: String,

    /// Step type
    pub step_type: StepType,

    /// Dependencies (step IDs that must complete first)
    pub dependencies: Vec<String>,

    /// Required capabilities
    pub capabilities: Vec<String>,

    /// Resource annotations
    pub resources: ResourceAnnotation,

    /// Failure policy
    pub failure_policy: FailurePolicy,

    /// Retry policy
    pub retry_policy: RetryPolicy,

    /// Timeout policy
    pub timeout_policy: TimeoutPolicy,
}

impl PlanStep {
    /// Create a new plan step
    pub fn new(id: impl Into<String>, step_type: StepType) -> Self {
        Self {
            id: id.into(),
            step_type,
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            resources: ResourceAnnotation::default(),
            failure_policy: FailurePolicy::default(),
            retry_policy: RetryPolicy::default(),
            timeout_policy: TimeoutPolicy::default(),
        }
    }

    /// Add a dependency
    pub fn depends_on(mut self, step_id: impl Into<String>) -> Self {
        self.dependencies.push(step_id.into());
        self
    }

    /// Add a required capability
    pub fn requires(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }
}

/// Type of plan step
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StepType {
    /// Tool execution
    Tool {
        name: String,
        version: String,
        input: serde_json::Value,
    },

    /// Observation
    Observation {
        source: String,
        filter: String,
    },

    /// Decision point
    Decision {
        condition: String,
        then_step: String,
        else_step: Option<String>,
    },

    /// Parallel execution
    Parallel {
        steps: Vec<String>,
    },

    /// Sequential execution
    Sequential {
        steps: Vec<String>,
    },

    /// Custom step type
    Custom {
        type_name: String,
        config: serde_json::Value,
    },
}

/// Resource requirements for a step
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResourceAnnotation {
    /// Maximum memory in bytes
    pub max_memory_bytes: Option<u64>,

    /// Maximum execution time in ms
    pub timeout_ms: u64,

    /// CPU units (abstract)
    pub cpu_units: Option<u64>,

    /// Whether this step requires exclusive access
    pub exclusive: bool,
}

impl Default for ResourceAnnotation {
    fn default() -> Self {
        Self {
            max_memory_bytes: None,
            timeout_ms: 30_000,
            cpu_units: None,
            exclusive: false,
        }
    }
}

/// Failure handling policy
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FailurePolicy {
    /// Stop the entire plan
    Stop,

    /// Continue to next step
    Continue,

    /// Retry with policy
    Retry,

    /// Run compensation
    Compensate { compensation_step: String },

    /// Fallback to alternative step
    Fallback { fallback_step: String },
}

impl Default for FailurePolicy {
    fn default() -> Self {
        Self::Stop
    }
}

/// Retry policy
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retries
    pub max_retries: u32,

    /// Backoff strategy
    pub backoff: BackoffStrategy,

    /// Retry on specific error types
    pub retry_on: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff: BackoffStrategy::Exponential { base_ms: 100, max_ms: 5000 },
            retry_on: Vec::new(),
        }
    }
}

/// Backoff strategy for retries
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed { delay_ms: u64 },

    /// Exponential backoff
    Exponential { base_ms: u64, max_ms: u64 },

    /// Linear backoff
    Linear { increment_ms: u64 },

    /// No delay
    None,
}

/// Timeout policy
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TimeoutPolicy {
    /// Timeout in milliseconds
    pub timeout_ms: u64,

    /// Action on timeout
    pub on_timeout: TimeoutAction,
}

impl Default for TimeoutPolicy {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            on_timeout: TimeoutAction::Error,
        }
    }
}

/// Action to take on timeout
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimeoutAction {
    /// Return an error
    Error,

    /// Use default value
    Default { value: serde_json::Value },

    /// Skip and continue
    Skip,
}
