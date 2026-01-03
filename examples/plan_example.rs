//! Example: Planning and DAG execution
//!
//! Demonstrates:
//! - Creating plans
//! - Compiling to DAG
//! - DAG validation
//! - Topological execution

use oracle_omen_plan::{
    compiler::PlanCompiler,
    dsl::{BackoffStrategy, FailurePolicy, Plan, PlanStep, ResourceAnnotation, StepType},
    validate::PlanValidator,
};

fn main() {
    println!("Oracle Omen - Planning Example");
    println!("==============================\n");

    // Create a plan with dependencies
    let mut plan = Plan::new("example_plan");

    // Step 1: Read config (no dependencies)
    plan.add_step(
        PlanStep::new(
            "read_config",
            StepType::Tool {
                name: "config_reader".to_string(),
                version: "1.0.0".to_string(),
                input: serde_json::json!({"path": "/etc/config.toml"}),
            },
        )
        .requires("fs:read:/etc"),
    );

    // Step 2: Fetch data (no dependencies, can run parallel with step 1)
    plan.add_step(
        PlanStep::new(
            "fetch_data",
            StepType::Tool {
                name: "http_fetch".to_string(),
                version: "1.0.0".to_string(),
                input: serde_json::json!({"url": "https://api.example.com/data"}),
            },
        )
        .requires("network:http:get"),
    );

    // Step 3: Process data (depends on both)
    plan.add_step(
        PlanStep::new(
            "process_data",
            StepType::Tool {
                name: "processor".to_string(),
                version: "1.0.0".to_string(),
                input: serde_json::json!({}),
            },
        )
        .depends_on("read_config")
        .depends_on("fetch_data"),
    );

    // Step 4: Write output (depends on process)
    plan.add_step(
        PlanStep::new(
            "write_output",
            StepType::Tool {
                name: "file_writer".to_string(),
                version: "1.0.0".to_string(),
                input: serde_json::json!({"path": "/tmp/output.json"}),
            },
        )
        .depends_on("process_data")
        .requires("fs:write:/tmp"),
    );

    println!("Plan: {}", plan.name);
    println!("Steps: {}\n", plan.len());

    // Validate plan
    match PlanValidator::validate(&plan) {
        Ok(()) => println!("Plan validation: PASSED\n"),
        Err(e) => {
            println!("Plan validation: FAILED - {}\n", e);
            return;
        }
    }

    // Compile to DAG
    let dag = match PlanCompiler::compile(&plan) {
        Ok(dag) => dag,
        Err(e) => {
            println!("Compilation failed: {}", e);
            return;
        }
    };

    println!("DAG compiled: {} nodes\n", dag.len());

    // Get topological order
    let order = dag.topological_order().unwrap();
    println!("Execution order:");
    for (i, node_id) in order.iter().enumerate() {
        println!("  {}. {}", i + 1, node_id);

        // Show dependencies
        let deps = dag.dependencies(node_id);
        if !deps.is_empty() {
            println!("     (depends on: {})", deps.iter().cloned().collect::<Vec<_>>().join(", "));
        }
    }
    println!();

    // Validate DAG
    match PlanValidator::validate_dag(&dag) {
        Ok(()) => println!("DAG validation: PASSED"),
        Err(e) => println!("DAG validation: FAILED - {}", e),
    }
}
