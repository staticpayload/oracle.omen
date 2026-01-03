//! Self-evolution patch system with governance.
//!
//! Patches are data, not code. All patches must:
//! - Be versioned
//! - Be signed
//! - Pass test gates
//! - Be auditable
//! - Preserve determinism

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod patch;
pub mod signature;
pub mod gate;
pub mod apply;
pub mod store;

pub use patch::*;
pub use signature::*;
pub use gate::*;
pub use apply::*;
pub use store::*;
