//! Example: Simple echo agent
//!
//! Demonstrates:
//! - Basic agent structure
//! - Tool usage
//! - Event logging
//! - Deterministic behavior

use oracle_omen_core::{
    capability::CapabilitySet,
    event::{Event, EventId, EventKind, EventPayload, LogicalTime},
    state::{AgentState, Decision, Observation, StateMachine, Transition},
};
use oracle_omen_runtime::tools::{EchoTool, ToolRegistry};
use std::sync::Arc;

/// Simple echo agent
struct EchoAgent;

impl StateMachine for EchoAgent {
    fn transition(
        &self,
        state: &AgentState,
        observation: &Observation,
        _tool_responses: &[oracle_omen_core::state::ToolResponse],
        _context: &oracle_omen_core::state::ExecutionContext,
    ) -> oracle_omen_core::state::StateResult<Transition> {
        // Get the observation data
        if let Some(input) = observation.data.get("input") {
            match input {
                oracle_omen_core::state::StateValue::String(s) => {
                    // Echo the input back
                    let decision = Decision::ToolCall(
                        oracle_omen_core::state::ToolCall::new("echo", "1.0.0", s.clone()),
                    );
                    let new_state = state.clone();
                    return Ok(Transition::new(new_state, decision));
                }
                _ => {}
            }
        }

        // Default: no action
        Ok(Transition::new(state.clone(), Decision::None))
    }

    fn initial_state(&self) -> AgentState {
        AgentState::with_run_id(1)
    }
}

fn main() {
    println!("Oracle Omen - Echo Agent Example");
    println!("================================\n");

    // Create agent
    let agent = EchoAgent;

    // Create tool registry with echo tool
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(EchoTool)).unwrap();

    // Create capability set (empty for echo - no capabilities needed)
    let capabilities = CapabilitySet::empty();

    // Run agent
    let state = agent.initial_state();
    println!("Initial state hash: {}\n", state.hash());

    // Create an observation
    let observation = Observation::new("user_input", 0)
        .with_field("input", oracle_omen_core::state::StateValue::String("Hello, World!".into()));

    println!("Observation: {:?}", observation);
    println!();

    // Process observation
    let context = oracle_omen_core::state::ExecutionContext::new(0, 1);
    match agent.transition(&state, &observation, &[], &context) {
        Ok(transition) => {
            println!("Transition hash: {}", transition.transition_hash);
            println!("New state hash: {}", transition.state.hash());
            println!("Decision: {:?}", transition.decision);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nCapabilities: {} granted", capabilities.len());
    println!("Available tools: {}", tools.list().len());
}
