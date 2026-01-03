//! WASM sandbox for tool execution.
//!
//! Provides:
//! - Fuel-limited execution
//! - Memory limits
//! - Deterministic result normalization
//! - Host function whitelisting

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod sandbox;
pub mod limits;
pub mod host;
pub mod compile;

pub use sandbox::*;
pub use limits::*;
pub use host::*;
pub use compile::*;
