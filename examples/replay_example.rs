//! Example: Event replay and divergence detection
//!
//! Demonstrates:
//! - Creating event logs
//! - Replay execution
//! - Divergence detection
//! - Snapshot usage

use oracle_omen_core::{
    event::{Event, EventId, EventKind, EventPayload, LogicalTime},
    hash::Hash,
    replay::{ReplayEngine, Snapshot, SnapshotManager},
    state::AgentState,
};

fn create_sample_log(run_id: u64, event_count: u64) -> oracle_omen_core::event::EventLog {
    use oracle_omen_core::event::EventLog;
    use std::collections::BTreeMap;

    let mut log = EventLog::new(run_id);

    // Add initial event
    let init_event = Event::new(
        EventId::initial(run_id),
        EventKind::AgentInit,
        LogicalTime::initial(run_id),
        EventPayload::AgentInit(oracle_omen_core::event::AgentInitPayload {
            agent_type: "test".to_string(),
            agent_version: "1.0.0".to_string(),
            config: BTreeMap::new(),
        }),
    );
    log.append(init_event).unwrap();

    // Add observations
    for i in 1..event_count {
        let event = Event::new(
            EventId::new(run_id, i),
            EventKind::Observation,
            LogicalTime::new(run_id, i),
            EventPayload::Observation(oracle_omen_core::event::ObservationPayload {
                obs_type: format!("observation_{}", i),
                data: {
                    let mut map = BTreeMap::new();
                    map.insert("value".to_string(), i.to_string());
                    map
                },
                source: "test".to_string(),
            }),
        );
        log.append(event).unwrap();
    }

    log
}

fn main() {
    println!("Oracle Omen - Replay Example");
    println!("============================\n");

    // Create two similar logs
    let log1 = create_sample_log(1, 10);
    let log2 = create_sample_log(2, 10);

    println!("Log 1: {} events", log1.len());
    println!("Log 2: {} events\n", log2.len());

    // Replay log 1
    let mut engine1 = ReplayEngine::new(log1.clone());
    let _state1 = engine1.replay_all().unwrap();
    println!("Replay 1 complete: {} events processed", engine1.position());

    // Verify
    let verification = engine1.verify().unwrap();
    println!("Verification: {}", verification);
    println!();

    // Create snapshot
    let snapshot = Snapshot::new("snapshot_5", 1, 5, engine1.current_state().clone());
    println!("Snapshot at position 5:");
    println!("  ID: {}", snapshot.id);
    println!("  Position: {}", snapshot.position);
    println!("  State hash: {}", snapshot.state_hash);
    println!("  Valid: {}", snapshot.verify());
    println!();

    // Snapshot manager
    let mut manager = SnapshotManager::new();
    manager.add(snapshot);

    println!("Snapshot positions: {:?}", manager.positions());
    println!();

    // Divergence detection (logs have different run_ids but same structure)
    let engine2 = ReplayEngine::new(log2);
    let divergences = engine1.detect_divergence(&engine2);

    if divergences.is_empty() {
        println!("No divergences detected!");
    } else {
        println!("Divergences:");
        for d in divergences {
            println!("  {}", d);
        }
    }
}
