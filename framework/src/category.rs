//! A list of separate, but related commands.

use crate::command::CommandId;

/// Grouping of independent commands with a related theme.
///
/// This grouping, or "categorization" is not special in any way and
/// does not affect invocation of commands. The type serves to simplify
/// [registration of commands][register] and displaying commands together in help messages.
///
/// [register]: crate::configuration::Configuration::command
#[derive(Debug)]
pub struct Category {
    /// Name of the category.
    pub name: String,
    /// [Command][cmd]s pertaining to this category.
    ///
    /// [cmd]: crate::command::Command
    pub commands: Vec<CommandId>,
}
