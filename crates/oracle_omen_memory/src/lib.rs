// oracle_omen_memory: CRDT, provenance, retrieval ordering
//
// Provides:
// - CRDT document store
// - Causal event linkage
// - Temporal queries
// - Deterministic retrieval

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod store;
pub mod document;
pub mod provenance;
pub mod query;

pub use store::*;
pub use document::*;
pub use provenance::*;
pub use query::*;
