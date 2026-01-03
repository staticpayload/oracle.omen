//! Policy language and compiler for capability governance.
//!
//! Policies define:
//! - What capabilities an agent may request
//! - What tools may be used
//! - What operations are permitted
//! - What self-modifications are allowed

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod lang;
pub mod compiler;
pub mod engine;
pub mod schema;

pub use lang::*;
pub use compiler::*;
pub use engine::*;
pub use schema::*;
