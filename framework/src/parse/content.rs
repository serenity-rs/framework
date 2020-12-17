//! Parsing iterators to parse the content out of the message.
//!
//! Parsing iterators are [`Iterator`]s that parse one group, or command, at a time.
//! These iterators will ensure that all groups and commands are valid. By "valid",
//! this means:
//!
//! - a prefix of a group, which exists in the [`GroupMap`] or is a subgroup of the previously
//! provided group, is provided, and/or
//! - a name of a command, which exists in the [`CommandMap`] or is a subcommand of the previously
//! provided command or is part of a group, is provided.
//!
//! An infinite amount of groups and commands can be provided by a user, so long
//! as each of them are valid.
//!
//! [`Iterator`]: std::iter::Iterator
//! [`GroupMap`]: crate::configuration::Configuration::groups
//! [`CommandMap`]: crate::configuration::Configuration::commands

use crate::command::Command;
use crate::configuration::Configuration;
use crate::error::DispatchError;
use crate::group::Group;
use crate::utils::Segments;

/// Group parsing iterator.
///
/// This is returned by [`groups`].
///
/// Refer to its documentation for more information.
///
/// [`groups`]: self::groups
pub struct GroupIterator<'a, 'b, 'c, D, E> {
    conf: &'a Configuration<D, E>,
    segments: &'b mut Segments<'c>,
    group: Option<&'a Group<D, E>>,
}

impl<'a, 'b, 'c, D, E> Iterator for GroupIterator<'a, 'b, 'c, D, E> {
    type Item = Result<Option<&'a Group<D, E>>, DispatchError>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.segments.current()?;

        if let Some(g) = self.conf.groups.get_by_name(&*name) {
            if let Some(group) = self.group {
                if !group.subgroups.contains(&g.id) {
                    return Some(Err(DispatchError::InvalidSubgroup(group.id, g.id)));
                }
            }

            self.segments.next();
            self.group = Some(g);
        }

        Some(Ok(self.group))
    }
}

/// Creates the group parsing iterator.
///
/// The [returned iterator][iter] will iterate through the segments of the message,
/// returning each valid group that it can find.
///
/// ## Return type of the iterator
///
/// The iterator will return items of the type `Result<Option<&`[`Group`]`>,`[`DispatchError`]`>`.
///
/// The `Option` signifies whether a group was present in the content or not.
/// `Result` signifies whether the current group belongs to the previous group, in case it does
/// not, the [`InvalidSubgroup`] error is returned.
///
/// The `Option` returned from calling [`Iterator::next`] will signify whether the content was
/// empty, or potentially had a group.
///
/// [iter]: self::GroupIterator
/// [`Group`]: crate::group::Group
/// [`DispatchError`]: crate::error::DispatchError
/// [`InvalidSubgroup`]: crate::error::DispatchError::InvalidSubgroup
pub fn groups<'a, 'b, 'c, D, E>(
    conf: &'a Configuration<D, E>,
    segments: &'b mut Segments<'c>,
) -> GroupIterator<'a, 'b, 'c, D, E> {
    GroupIterator {
        conf,
        segments,
        group: None,
    }
}

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
    group: Option<&'a Group<D, E>>,
    command: Option<&'a Command<D, E>>,
    beginning: bool,
}

impl<'a, 'b, 'c, D, E> Iterator for CommandIterator<'a, 'b, 'c, D, E> {
    type Item = Result<&'a Command<D, E>, DispatchError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.beginning {
            let default_command = default_command(self.conf, self.group);
            let command = match initial_command(self.conf, self.segments) {
                Ok(cmd) => cmd,
                Err(e) => match default_command.ok_or(e) {
                    Ok(cmd) => cmd,
                    e @ Err(_) => return Some(e),
                },
            };

            if let Some(group) = self.group {
                if !group.commands.contains(&command.id) {
                    return Some(Err(DispatchError::InvalidCommand(group.id, command.id)));
                }
            }

            self.beginning = false;

            self.command = Some(command);

            return Some(Ok(command));
        }

        let name = self.segments.current()?;

        if let Some(cmd) = self.conf.commands.get_by_name(&*name) {
            if let Some(command) = self.command {
                if !command.subcommands.contains(&cmd.id) {
                    return Some(Err(DispatchError::InvalidSubcommand(command.id, cmd.id)));
                }
            }

            self.segments.next();
            self.command = Some(cmd);
        }

        // It is an error to be without a command at this point.
        // This is handled above in the "beginning" stage.
        // `self.command` must be `Some` once this line of code
        // gets executed, hence we `unwrap()`.
        Some(Ok(self.command.unwrap()))
    }
}

/// Creates the command parsing iterator.
///
/// The [returned iterator][iter] will iterate through the segments of the message,
/// returning each valid command that it can find.
///
/// ## Return type of the iterator
///
/// The iterator will return items of the type `Result<&`[`Command`]`,`[`DispatchError`]`>`.
///
/// The `Result` signifies whether the current command belongs to the previous command.
/// In case it does not, the [`InvalidSubcommand`] error is returned.
///
/// The `Option` returned from calling [`Iterator::next`] will signify whether the content was
/// empty, or had a command. However, in case the content is empty for the first command,
/// the [`MissingContent`] error is returned, as at least one command is necessary for a valid
/// invocation. Additionally, if a group is given and the command does not belong to it,
/// the [`InvalidCommand`] error is returned.
///
/// [iter]: self::CommandIterator
/// [`Command`]: crate::command::Command
/// [`DispatchError`]: crate::error::DispatchError
/// [`InvalidSubcommand`]: crate::error::DispatchError::InvalidSubcommand
/// [`MissingContent`]: crate::error::DispatchError::MissingContent
/// [`InvalidCommand`]: crate::error::DispatchError::InvalidCommand
pub fn commands<'a, 'b, 'c, D, E>(
    conf: &'a Configuration<D, E>,
    segments: &'b mut Segments<'c>,
    group: Option<&'a Group<D, E>>,
) -> CommandIterator<'a, 'b, 'c, D, E> {
    CommandIterator {
        conf,
        segments,
        group,
        command: None,
        beginning: true,
    }
}

/// Finds the command instance for the default command of a group.
fn default_command<'a, D, E>(
    conf: &'a Configuration<D, E>,
    group: Option<&Group<D, E>>,
) -> Option<&'a Command<D, E>> {
    group
        .and_then(|g| g.default_command)
        .map(|id| &conf.commands[id])
}

/// Finds the first command in the content.
///
/// # Errors
///
/// - [`MissingContent`] is returned if there is no more content.
/// - [`InvalidCommandName`] is returned if there was a segment with a wrong command name.
///
/// [`MissingContent`]: crate::error::DispatchError::MissingContent
/// [`InvalidCommandName`]: crate::error::DispatchError::InvalidCommandName
fn initial_command<'a, D, E>(
    conf: &'a Configuration<D, E>,
    segments: &mut Segments<'_>,
) -> Result<&'a Command<D, E>, DispatchError> {
    match segments.current() {
        Some(name) => match conf.commands.get_by_name(&*name) {
            Some(cmd) => Ok(cmd),
            None => Err(DispatchError::InvalidCommandName(name.into_owned())),
        },
        None => Err(DispatchError::MissingContent),
    }
}
