//! Iterators to parse the content out of the message.

use crate::command::Command;
use crate::configuration::Configuration;
use crate::error::DispatchError;
use crate::utils::Segments;

/// Command parsing iterator.
///
/// This is returned by [`commands`].
///
/// Refer to its documentation for more information.
///
/// [`commands`]: self::commands
pub struct CommandIterator<'a, 'b, 'c, D, E> {
    conf: &'a Configuration<D, E>,
    segments: &'b mut Segments<'c>,
    command: Option<&'a Command<D, E>>,
}

impl<'a, 'b, 'c, D, E> Iterator for CommandIterator<'a, 'b, 'c, D, E> {
    type Item = Result<&'a Command<D, E>, DispatchError>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.segments.current()?;

        let cmd = match self.conf.commands.get_by_name(&*name) {
            Some(cmd) => {
                self.segments.next();
                cmd
            },
            None => return Some(Err(DispatchError::InvalidCommandName(name.into_owned()))),
        };

        if let Some(command) = self.command {
            if !command.subcommands.contains(&cmd.id) {
                return Some(Err(DispatchError::InvalidSubcommand(command.id, cmd.id)));
            }
        }

        self.command = Some(cmd);

        Some(Ok(cmd))
    }
}

/// Creates a command parsing iterator.
///
/// The [returned iterator][iter] will iterate through the segments of the message,
/// returning each valid command that it can find.
///
/// ## Return type of the iterator
///
/// The iterator will return items of the type `Result<&`[`Command`]`,`[`DispatchError`]`>`.
///
/// The `Result` signifies whether a given name for a command was existed and the the command
/// belongs to the previous parsed command.
///
/// If the first case is not satisfied, the [`InvalidCommandName`] error is returned. On the second
/// case, the [`InvalidSubcommand`] error is returned.
///
/// The `Option` returned from calling [`Iterator::next`] will signify whether the content had a
/// command, or was empty.
///
/// [iter]: self::CommandIterator
/// [`Command`]: crate::command::Command
/// [`DispatchError`]: crate::error::DispatchError
/// [`InvalidCommandName`]: crate::error::DispatchError::InvalidCommandName
/// [`InvalidSubcommand`]: crate::error::DispatchError::InvalidSubcommand
pub fn commands<'a, 'b, 'c, D, E>(
    conf: &'a Configuration<D, E>,
    segments: &'b mut Segments<'c>,
) -> CommandIterator<'a, 'b, 'c, D, E> {
    CommandIterator {
        conf,
        segments,
        command: None,
    }
}
