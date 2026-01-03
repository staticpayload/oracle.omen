//! Example: Memory store with CRDT and provenance
//!
//! Demonstrates:
//! - CRDT document storage
//! - Provenance tracking
//! - Temporal queries
//! - Deterministic retrieval

use oracle_omen_memory::{
    document::{Document, DocumentValue},
    provenance::{Operation, ProvenanceRecord, ProvenanceTracker},
    query::{QueryBuilder, QueryOrder},
    store::MemoryStore,
};

fn main() {
    println!("Oracle Omen - Memory Store Example");
    println!("==================================\n");

    let mut store = MemoryStore::new();
    let mut tracker = ProvenanceTracker::new();

    // Write documents
    for i in 1..=5 {
        let doc = Document::new(
            format!("key_{}", i),
            DocumentValue::Integer(i * 10),
            i, // causal_event
        );
        store.write(doc);

        tracker.record(ProvenanceRecord::new(i, Operation::Write, format!("key_{}", i)));
    }

    println!("Store contains {} documents", store.len());
    println!("Store hash: {}\n", store.hash());

    // Read a document
    if let Some(doc) = store.read("key_3") {
        println!("Document 'key_3':");
        println!("  Value: {:?}", doc.value);
        println!("  Causal event: {}", doc.causal_event);
        println!("  Hash: {}", doc.hash());
        println!();
    }

    // Provenance tracking
    println!("Events affecting 'key_2':");
    for event_id in tracker.history_for_key("key_2") {
        println!("  Event {}", event_id);
    }
    println!();

    // Deterministic retrieval (sorted keys)
    println!("All keys (deterministic order):");
    for key in store.keys_sorted() {
        println!("  {}", key);
    }
    println!();

    // Query with filters
    println!("Query for keys with prefix 'key_':");
    let results = QueryBuilder::new(&store)
        .filter(oracle_omen_memory::query::Filter::KeyPrefix("key_".to_string()))
        .order_by(QueryOrder::Key)
        .execute()
        .unwrap();

    for item in results {
        println!("  {} -> {:?}", item.key, item.value);
    }
    println!();

    // Snapshot
    let snapshot = store.snapshot();
    println!("Snapshot:");
    println!("  Documents: {}", snapshot.document_hashes.len());
    println!("  Store hash: {}", snapshot.store_hash);
}
