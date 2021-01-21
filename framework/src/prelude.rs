//! A series of re-exports to simplify usage of the framework.
//!
//! Some exports are renamed to avoid name conflicts as they are generic.
//! These include:
//!
//! - `Context` -> `FrameworkContext`
//! - `Error` -> `FrameworkError`

#[cfg(feature = "macros")]
pub use command_attr::{check, command, hook};

pub use crate::category::Category;
pub use crate::check::{Check, CheckResult, Reason};
pub use crate::command::{Command, CommandResult};
pub use crate::configuration::Configuration;
pub use crate::context::{CheckContext, Context as FrameworkContext};
pub use crate::error::{DispatchError, Error as FrameworkError};
pub use crate::Framework;
