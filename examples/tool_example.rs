//! Example: Creating custom tools
//!
//! Demonstrates:
//! - Tool trait implementation
//! - Capability requirements
//! - Resource bounds
//! - Response normalization

use oracle_omen_core::{
    capability::Capability,
    hash::Hash,
    tool::{Determinism, ResourceBounds, SideEffect, ToolError, ToolId, ToolResult},
};
use oracle_omen_runtime::tools::{DynTool, ToolMetadata};
use serde::{Deserialize, Serialize};

/// Custom calculation tool
#[derive(Clone)]
struct CalculatorTool;

impl DynTool for CalculatorTool {
    fn id(&self) -> &ToolId {
        static ID: ToolId = ToolId::new("calculator", "1.0.0");
        &ID
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["compute:arithmetics".to_string()]
    }

    fn side_effects(&self) -> SideEffect {
        SideEffect::Pure
    }

    fn resource_bounds(&self) -> &ResourceBounds {
        static BOUNDS: ResourceBounds = ResourceBounds::with_timeout(1000);
        &BOUNDS
    }

    fn execute(&self, input: &[u8], _metadata: &ToolMetadata) -> ToolResult<Vec<u8>> {
        // Parse input
        let request: CalcRequest = serde_json::from_slice(input)
            .map_err(|e| ToolError::InvalidInput {
                tool: "calculator".to_string(),
                reason: e.to_string(),
            })?;

        // Perform calculation
        let result = match request.op {
            Op::Add => request.a + request.b,
            Op::Sub => request.a - request.b,
            Op::Mul => request.a * request.b,
            Op::Div => {
                if request.b == 0 {
                    return Err(ToolError::ExecutionFailed {
                        tool: "calculator".to_string(),
                        reason: "Division by zero".to_string(),
                    });
                }
                request.a / request.b
            }
        };

        // Serialize response
        let response = CalcResponse { result };
        serde_json::to_vec(&response).map_err(|e| ToolError::SerializationFailed {
            tool: "calculator".to_string(),
            reason: e.to_string(),
        })
    }

    fn input_schema(&self) -> &str {
        r#"{"type": "object", "properties": {"a": {"type": "integer"}, "b": {"type": "integer"}, "op": {"type": "string"}}}"#
    }

    fn output_schema(&self) -> &str {
        r#"{"type": "object", "properties": {"result": {"type": "integer"}}}"#
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CalcRequest {
    a: i64,
    b: i64,
    op: Op,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CalcResponse {
    result: i64,
}

fn main() {
    println!("Oracle Omen - Custom Tool Example");
    println!("=================================\n");

    let tool = CalculatorTool;
    let metadata = ToolMetadata {
        logical_time: 0,
        run_id: 1,
        seed: None,
    };

    println!("Tool ID: {}", tool.id());
    println!("Capabilities: {:?}", tool.capabilities());
    println!("Side effects: {:?}", tool.side_effects());
    println!("Determinism: {:?}", tool.determinism());
    println!("Resource bounds: {:?}", tool.resource_bounds());
    println!();

    // Example: 15 + 27 = 42
    let request = CalcRequest {
        a: 15,
        b: 27,
        op: Op::Add,
    };

    let input = serde_json::to_vec(&request).unwrap();
    println!("Input: {}", String::from_utf8_lossy(&input));

    match tool.execute(&input, &metadata) {
        Ok(output) => {
            let response: CalcResponse = serde_json::from_slice(&output).unwrap();
            println!("Result: {}", response.result);
        }
        Err(e) => println!("Error: {}", e),
    }
}
