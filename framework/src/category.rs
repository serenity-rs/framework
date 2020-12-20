//! A list of separate, but related commands.

use crate::command::{CommandConstructor, CommandId};

/// Grouping of independent commands with a related theme.
///
/// This grouping, or "categorization" is not special in any way and
/// does not affect invocation of commands. The type serves to simplify
/// [registration of commands][register] and displaying commands together in help messages.
///
/// [register]: crate::configuration::Configuration::command
#[derive(Debug, Default, Clone)]
pub struct Category {
    /// Name of the category.
    pub name: String,
    /// [`Command`][cmd]s pertaining to this category.
    ///
    /// [cmd]: crate::command::Command
    pub commands: Vec<CommandId>,
}

/// A builder type for creating a [`Category`] from scratch.
#[derive(Debug, Default, Clone)]
pub struct CategoryBuilder {
    inner: Category,
}

impl CategoryBuilder {
    /// Constructs a new instance of the builder.
    ///
    /// Argument is the name of the category.
    pub fn new(name: &str) -> Self {
        Self {
            inner: Category {
                name: name.to_string(),
                ..Category::default()
            },
        }
    }

    /// Assigns a command to this category.
    ///
    /// The command is added to the [`commands`] list.
    ///
    /// [`commands`]: Category::commands
    pub fn command<D, E>(&mut self, command: CommandConstructor<D, E>) -> &mut Self {
        self.inner.commands.push(CommandId::from(command));

        self
    }

    /// Complete building a category.
    pub fn build(self) -> Category {
        self.inner
    }
}
