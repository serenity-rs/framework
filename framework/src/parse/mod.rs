//! Group of functions that take part in the parsing stage.
//!
//! Usable outside of the framework.

pub mod content;
pub mod prefix;

pub use content::{commands, groups};
pub use prefix::content;
