# Planning and DAG Execution

## Overview

Agents use plans to describe complex work. Plans are compiled to DAGs for execution.

## Planning DSL

### Plan Structure

```rust
pub struct Plan {
    pub name: String,
    pub steps: Vec<PlanStep>,
    pub metadata: BTreeMap<String, String>,
}
```

### Plan Step

```rust
pub struct PlanStep {
    pub id: String,
    pub step_type: StepType,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub resources: ResourceAnnotation,
    pub failure_policy: FailurePolicy,
    pub retry_policy: RetryPolicy,
    pub timeout_policy: TimeoutPolicy,
}
```

## Step Types

| Type | Description |
|------|-------------|
| `Tool` | Execute a tool |
| `Observation` | Read from environment |
| `Decision` | Branch based on condition |
| `Parallel` | Execute steps concurrently |
| `Sequential` | Execute steps in order |
| `Custom` | User-defined step type |

## Failure Policies

| Policy | Behavior |
|--------|----------|
| `Stop` | Stop entire plan |
| `Continue` | Continue to next step |
| `Retry` | Retry with policy |
| `Compensate` | Run compensation step |
| `Fallback` | Run alternative step |

## DAG Compilation

```rust
let plan = create_plan();
let dag = PlanCompiler::compile(&plan)?;
```

Compilation validates:
- Step IDs are unique
- Dependencies exist
- No circular dependencies
- Resource bounds are sensible

## Topological Ordering

The DAG is executed in topological order:

```
   A
  / \
 B   C
  \ /
   D

Order: A -> B -> C -> D
or:    A -> C -> B -> D  (B and C can run in parallel)
```

## Scheduler

The scheduler manages execution:

```rust
let mut scheduler = Scheduler::new(max_concurrent: 4);
scheduler.initialize(&dag);

while !scheduler.is_complete() {
    if let Some(node_id) = scheduler.next() {
        // Execute node
        executor.execute_node(&dag, &node_id).await?;
        scheduler.complete(&node_id)?;
    }
}
```

## Backpressure

The scheduler enforces limits:
- Maximum concurrent tasks
- Resource limits (memory, CPU)
- Queue depth

## Invariants

1. **No cycles**: DAG is acyclic
2. **Dependency satisfaction**: Dependencies run first
3. **Deterministic ordering**: Same DAG produces same execution order
4. **Resource enforcement**: Bounds are never exceeded
