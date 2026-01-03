//! Directed Acyclic Graph (DAG) representation.
//!
//! Compiled form of a Plan with validated dependencies.

use std::collections::{BTreeMap, BTreeSet, HashSet};

/// A DAG representing a compiled plan
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Dag {
    /// DAG name
    pub name: String,

    /// DAG nodes
    nodes: BTreeMap<String, DagNode>,

    /// Edges: node -> dependencies
    edges: BTreeMap<String, BTreeSet<String>>,

    /// Reverse edges: node -> dependents
    reverse_edges: BTreeMap<String, BTreeSet<String>>,
}

impl Dag {
    /// Create a new DAG
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            reverse_edges: BTreeMap::new(),
        }
    }

    /// Add a node to the DAG
    pub fn add_node(&mut self, node: DagNode) -> Result<(), DagError> {
        let id = node.id.clone();
        if self.nodes.contains_key(&id) {
            return Err(DagError::DuplicateNode(id));
        }
        self.nodes.insert(id.clone(), node);
        self.edges.insert(id.clone(), BTreeSet::new());
        self.reverse_edges.insert(id, BTreeSet::new());
        Ok(())
    }

    /// Add an edge (dependency) between nodes
    pub fn add_edge(&mut self, from: String, to: String) -> Result<(), DagError> {
        if !self.nodes.contains_key(&from) {
            return Err(DagError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(DagError::NodeNotFound(to));
        }

        // Check for cycle
        if self.would_create_cycle(&from, &to) {
            return Err(DagError::CycleDetected { from, to });
        }

        self.edges.entry(from.clone()).or_default().insert(to.clone());
        self.reverse_edges.entry(to).or_default().insert(from);
        Ok(())
    }

    /// Check if adding an edge would create a cycle
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        self.has_path(to, from, &mut visited)
    }

    /// Check if there's a path from start to end
    fn has_path(&self, start: &str, end: &str, visited: &mut HashSet<String>) -> bool {
        if start == end {
            return true;
        }
        if !visited.insert(start.to_string()) {
            return false;
        }
        if let Some(deps) = self.edges.get(start) {
            for dep in deps {
                if self.has_path(dep, end, visited) {
                    return true;
                }
            }
        }
        false
    }

    /// Get topological ordering of nodes
    pub fn topological_order(&self) -> Result<Vec<String>, DagError> {
        let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
        for node in self.nodes.keys() {
            in_degree.insert(node.clone(), 0);
        }
        for deps in self.edges.values() {
            for dep in deps {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(n, _)| n.clone())
            .collect();
        queue.sort(); // Ensure deterministic ordering

        let mut result = Vec::new();
        while let Some(node) = queue.pop() {
            result.push(node.clone());
            if let Some(deps) = self.reverse_edges.get(&node) {
                for dependent in deps {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(dependent.clone());
                        }
                    }
                }
                queue.sort();
            }
        }

        if result.len() != self.nodes.len() {
            return Err(DagError::CycleDetected {
                from: "unknown".to_string(),
                to: "unknown".to_string(),
            });
        }

        Ok(result)
    }

    /// Get node by ID
    pub fn node(&self, id: &str) -> Option<&DagNode> {
        self.nodes.get(id)
    }

    /// Get all nodes
    pub fn nodes(&self) -> &BTreeMap<String, DagNode> {
        &self.nodes
    }

    /// Get dependencies for a node
    pub fn dependencies(&self, id: &str) -> &BTreeSet<String> {
        self.edges.get(id).map_or(&BTreeSet::new(), |s| s)
    }

    /// Get dependents of a node
    pub fn dependents(&self, id: &str) -> &BTreeSet<String> {
        self.reverse_edges.get(id).map_or(&BTreeSet::new(), |s| s)
    }

    /// Get node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Validate the DAG structure
    pub fn validate(&self) -> Result<(), DagError> {
        // Check for cycles
        self.topological_order()?;

        // Check all dependencies exist
        for (node, deps) in &self.edges {
            for dep in deps {
                if !self.nodes.contains_key(dep) {
                    return Err(DagError::DependencyNotFound {
                        node: node.clone(),
                        dependency: dep.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// A node in the DAG
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DagNode {
    /// Unique node identifier
    pub id: String,

    /// Node type
    pub node_type: DagNodeType,

    /// Required capabilities
    pub capabilities: Vec<String>,

    /// Resource bounds
    pub resources: ResourceAnnotation,

    /// Failure policy
    pub failure_policy: FailurePolicy,

    /// Retry policy
    pub retry_policy: RetryPolicy,

    /// Timeout policy
    pub timeout_policy: TimeoutPolicy,

    /// Node metadata
    pub metadata: BTreeMap<String, String>,
}

impl DagNode {
    /// Create a new DAG node
    pub fn new(id: impl Into<String>, node_type: DagNodeType) -> Self {
        Self {
            id: id.into(),
            node_type,
            capabilities: Vec::new(),
            resources: ResourceAnnotation::default(),
            failure_policy: FailurePolicy::default(),
            retry_policy: RetryPolicy::default(),
            timeout_policy: TimeoutPolicy::default(),
            metadata: BTreeMap::new(),
        }
    }
}

/// Type of DAG node
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DagNodeType {
    /// Execute a tool
    Tool { name: String, version: String },

    /// Make an observation
    Observation { source: String },

    /// Make a decision
    Decision { condition: String },

    /// Wait/delay
    Wait { duration_ms: u64 },

    /// Custom node type
    Custom { type_name: String, config: serde_json::Value },
}

/// DAG errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DagError {
    /// Node already exists
    DuplicateNode(String),

    /// Node not found
    NodeNotFound(String),

    /// Dependency not found
    DependencyNotFound { node: String, dependency: String },

    /// Cycle detected
    CycleDetected { from: String, to: String },

    /// Invalid structure
    InvalidStructure(String),
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DagError::DuplicateNode(id) => write!(f, "Duplicate node: {}", id),
            DagError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            DagError::DependencyNotFound { node, dependency } => {
                write!(f, "Dependency '{}' of node '{}' not found", dependency, node)
            }
            DagError::CycleDetected { from, to } => {
                write!(f, "Cycle detected: edge {} -> {} would create cycle", from, to)
            }
            DagError::InvalidStructure(msg) => write!(f, "Invalid DAG structure: {}", msg),
        }
    }
}

impl std::error::Error for DagError {}

// Re-export types from dsl module
use crate::dsl::{
    BackoffStrategy, FailurePolicy, ResourceAnnotation, RetryPolicy, TimeoutAction, TimeoutPolicy,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_creation() {
        let mut dag = Dag::new("test");
        dag.add_node(DagNode::new("a", DagNodeType::Tool {
            name: "tool1".to_string(),
            version: "1.0.0".to_string(),
        }))
        .unwrap();
        dag.add_node(DagNode::new("b", DagNodeType::Tool {
            name: "tool2".to_string(),
            version: "1.0.0".to_string(),
        }))
        .unwrap();

        assert_eq!(dag.len(), 2);
    }

    #[test]
    fn test_dag_cycle_detection() {
        let mut dag = Dag::new("test");
        dag.add_node(DagNode::new("a", DagNodeType::Tool {
            name: "tool1".to_string(),
            version: "1.0.0".to_string(),
        }))
        .unwrap();
        dag.add_node(DagNode::new("b", DagNodeType::Tool {
            name: "tool2".to_string(),
            version: "1.0.0".to_string(),
        }))
        .unwrap();

        // Add edge a -> b
        dag.add_edge("a".to_string(), "b".to_string()).unwrap();

        // Try to add edge b -> a (would create cycle)
        assert!(dag.add_edge("b".to_string(), "a".to_string()).is_err());
    }

    #[test]
    fn test_topological_order() {
        let mut dag = Dag::new("test");
        dag.add_node(DagNode::new("a", DagNodeType::Tool {
            name: "tool1".to_string(),
            version: "1.0.0".to_string(),
        }))
        .unwrap();
        dag.add_node(DagNode::new("b", DagNodeType::Tool {
            name: "tool2".to_string(),
            version: "1.0.0".to_string(),
        }))
        .unwrap();
        dag.add_node(DagNode::new("c", DagNodeType::Tool {
            name: "tool3".to_string(),
            version: "1.0.0".to_string(),
        }))
        .unwrap();

        dag.add_edge("a".to_string(), "b".to_string()).unwrap();
        dag.add_edge("b".to_string(), "c".to_string()).unwrap();

        let order = dag.topological_order().unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }
}
