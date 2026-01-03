//! Scheduler for DAG execution with backpressure.

use crate::executor::{ExecResult, ExecError};
use oracle_omen_plan::dag::Dag;
use std::collections::{HashMap, VecDeque};

/// Scheduler for DAG execution
pub struct Scheduler {
    /// Ready to execute (dependencies satisfied)
    ready: VecDeque<String>,

    /// Pending (waiting for dependencies)
    pending: HashMap<String, Vec<String>>,

    /// Currently executing
    running: HashMap<String, RunningTask>,

    /// Maximum concurrent tasks
    max_concurrent: usize,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            ready: VecDeque::new(),
            pending: HashMap::new(),
            running: HashMap::new(),
            max_concurrent,
        }
    }

    /// Initialize scheduler with a DAG
    pub fn initialize(&mut self, dag: &Dag) -> ExecResult<()> {
        let order = dag
            .topological_order()
            .map_err(|e| ExecError::InvalidState(e.to_string()))?;

        // Track remaining dependencies for each node
        let mut remaining_deps: HashMap<String, usize> = HashMap::new();

        for node_id in &order {
            let deps = dag.dependencies(node_id);
            let dep_count = deps.as_ref().map_or(0, |d| d.len());
            remaining_deps.insert(node_id.clone(), dep_count);

            if let Some(dep_set) = deps {
                if dep_set.is_empty() {
                    self.ready.push_back(node_id.clone());
                } else {
                    self.pending.insert(node_id.clone(), dep_set.iter().cloned().collect());
                }
            } else {
                // No dependencies
                self.ready.push_back(node_id.clone());
            }
        }

        Ok(())
    }

    /// Get next task to execute
    pub fn next(&mut self) -> Option<String> {
        if self.running.len() >= self.max_concurrent {
            None
        } else {
            self.ready.pop_front()
        }
    }

    /// Mark a task as started
    pub fn start(&mut self, node_id: String, _task: RunningTask) {
        self.running.insert(node_id, _task);
    }

    /// Mark a task as complete
    pub fn complete(&mut self, node_id: &str) -> ExecResult<()> {
        self.running.remove(node_id);

        // Update pending tasks that depend on this one
        let mut newly_ready = Vec::new();
        for (pending_id, deps) in self.pending.iter_mut() {
            if let Some(pos) = deps.iter().position(|d| d == node_id) {
                deps.remove(pos);
                if deps.is_empty() {
                    newly_ready.push(pending_id.clone());
                }
            }
        }

        // Remove newly ready from pending and add to ready
        for id in newly_ready {
            self.pending.remove(&id);
            self.ready.push_back(id);
        }

        Ok(())
    }

    /// Check if scheduling is complete
    pub fn is_complete(&self) -> bool {
        self.ready.is_empty() && self.running.is_empty() && self.pending.is_empty()
    }

    /// Get number of running tasks
    pub fn running_count(&self) -> usize {
        self.running.len()
    }

    /// Get number of ready tasks
    pub fn ready_count(&self) -> usize {
        self.ready.len()
    }
}

/// A running task
#[derive(Clone, Debug)]
pub struct RunningTask {
    /// Node ID
    pub node_id: String,

    /// Start time (logical)
    pub start_time: u64,
}

impl RunningTask {
    /// Create a new running task
    pub fn new(node_id: impl Into<String>, start_time: u64) -> Self {
        Self {
            node_id: node_id.into(),
            start_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oracle_omen_plan::dag::{Dag, DagNode, DagNodeType};

    #[test]
    fn test_scheduler_initialization() {
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

        let mut scheduler = Scheduler::new(2);
        scheduler.initialize(&dag).unwrap();

        // Both nodes should be ready (no dependencies)
        assert_eq!(scheduler.ready_count(), 2);
    }

    #[test]
    fn test_scheduler_with_dependencies() {
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
        dag.add_edge("a".to_string(), "b".to_string()).unwrap();

        let mut scheduler = Scheduler::new(2);
        scheduler.initialize(&dag).unwrap();

        // Only 'a' should be ready
        assert_eq!(scheduler.ready_count(), 1);
        assert_eq!(scheduler.next(), Some("a".to_string()));

        // Complete 'a'
        scheduler.start("a".to_string(), RunningTask::new("a", 0));
        scheduler.complete("a").unwrap();

        // Now 'b' should be ready
        assert_eq!(scheduler.next(), Some("b".to_string()));
    }

    #[test]
    fn test_scheduler_backpressure() {
        let mut dag = Dag::new("test");
        for i in 0..5 {
            dag.add_node(DagNode::new(
                format!("node{}", i),
                DagNodeType::Tool {
                    name: format!("tool{}", i),
                    version: "1.0.0".to_string(),
                },
            ))
            .unwrap();
        }

        let mut scheduler = Scheduler::new(2);
        scheduler.initialize(&dag).unwrap();

        // Can only get 2 tasks (max_concurrent)
        assert!(scheduler.next().is_some());
        assert!(scheduler.next().is_some());
        assert!(scheduler.next().is_none()); // Backpressure
    }
}
