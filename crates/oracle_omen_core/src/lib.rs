// oracle_omen_core: Pure logic only, no IO
//
// Core abstractions for deterministic agent systems:
// - Event types and log schema
// - Stable hashing
// - State machine definitions
// - Capability types
// - Error types

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod event;
pub mod hash;
pub mod state;
pub mod capability;
pub mod tool;
pub mod error;
pub mod time;
pub mod serde_utils;
pub mod replay;

pub use event::*;
pub use hash::*;
pub use state::*;
pub use capability::*;
pub use tool::*;
pub use error::*;
pub use time::*;
pub use serde_utils::*;
pub use replay::*;
