//! Declarative CLI layer. A single registry of command
//! specifications drives parsing, help generation and
//! dispatch; handlers own behavior and stay independent
//! of parsing and translation.
pub mod context;
pub mod handlers;
pub mod help;
pub mod parser;
pub mod registry;

pub use context::{Invocation, Options};
pub use parser::parse;
