// Example: Patch proposal and application

use oracle_omen_core::{hash::Hash, state::AgentState};
use oracle_omen_patches::{
    apply::PatchEngine,
    gate::{ApprovalGate, DeterminismTestRunner, GateResult},
    patch::{Patch, PatchId, PatchStatus, PatchType, PatchTarget},
    signature::{KeyPair, Signature, SignedPatch, SignerId},
    signature,
    store::PatchStore,
};

fn main() {
    println!("Oracle Omen - Patch Example");
    println!("============================\n");

    // Create a patch
    let patch_id = PatchId::new(1, 0);
    let mut patch = Patch::new(
        patch_id.clone(),
        PatchType::Prompt,
        PatchTarget::SystemPrompt,
        "Update system prompt for clarity".to_string(),
    )
    .with_data("prompt", "You are a helpful assistant that provides accurate, concise answers.");

    // Add test requirements
    use oracle_omen_patches::patch::{TestRequirement, TestType, TestOutcome};
    patch.tests.push(TestRequirement {
        name: "determinism_check".to_string(),
        test_type: TestType::Determinism,
        expected: TestOutcome::Pass,
    });

    println!("Patch created: {:?}", patch.patch_type);
    println!("Target: {:?}", patch.target);
    println!("Reasoning: {}", patch.reasoning);
    println!("Tests: {}\n", patch.tests.len());

    // Create keypair for signing
    let keypair = KeyPair::generate();
    let signer = keypair.signer_id();
    let signature = keypair.sign(&patch.hash().to_hex().into_bytes());

    // Create signed patch
    let signed = SignedPatch::new(patch.clone(), signature, signer.clone());
    println!("Signature verified: {}\n", signed.verify());

    // Create patch store
    let mut store = PatchStore::new();
    store
        .add_patch(patch_id.to_string(), patch.clone(), PatchStatus::Proposed)
        .unwrap();

    // Create patch engine
    let mut engine = PatchEngine::new(store);

    // Run test gate
    let test_runner = DeterminismTestRunner;
    let test_result = engine.test_gate(&patch_id.to_string(), &test_runner).unwrap();
    println!("Test gate: {}", if test_result.is_passed() { "PASSED" } else { "FAILED" });

    // Approve the patch
    let approval_gate = ApprovalGate::new(vec![signer]);
    let approval_result = approval_gate.evaluate(&signature, &signer);
    println!("Approval gate: {}", if approval_result.is_passed() { "GRANTED" } else { "DENIED" });

    engine
        .approve(&patch_id.to_string(), signature, signer)
        .unwrap();

    // Apply the patch
    let mut state = AgentState::initial();
    println!("\nBefore patch:");
    println!("  State hash: {}", state.hash());

    let apply_result = engine.apply(&patch_id.to_string(), &mut state);
    println!("\nAfter patch:");
    println!("  State hash: {}", state.hash());
    println!("  Changes: {:?}", apply_result.unwrap().changes_made);
}
