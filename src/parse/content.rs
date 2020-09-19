//! Functions to parse the content out of the message.
//!
//! A valid content is a group, its subgroups (and their subgroups...), the group's command,
//! and its subcommands (and their subcommands...). A group need not be present in the content
//! if it does not possess prefixes. A prefixless group is considered a Top Level Group, which
//! is invisible to the user invoking a command, and only applies at the beginning of the content.
//! They cannot have subgroups, which must have prefixes.

use crate::command::Command;
use crate::configuration::Configuration;
use crate::error::{DispatchError, Error};
use crate::group::Group;
use crate::utils::Segments;

use std::future::Future;

/// Parses a [`Group`] out of the content.
///
/// If no groups are present in the content, `None` is returned.
/// Otherwise, the last group is returned.
///
/// A content can have an infinite amount of groups, so long they
/// have prefixes and are subgroups of the groups preceeding them.
///
/// A Top Level Group cannot be returned from this function,
/// as they do not have prefixes and subgroups.
///
/// As the function successfully parses a group, the given `f` async function
/// is called with the group. Its purpose is to short-circuit this function if
/// the group does not conform to certain conditions, such as its checks failing.
/// If `f` returns an error, it is propagated to the caller of this function.
///
/// [`Group`]: ../../group/struct.Group.html
pub async fn group<'a, D, E, F, Fut>(
    conf: &'a Configuration<D, E>,
    segments: &mut Segments<'_>,
    f: F,
) -> Result<Option<&'a Group<D, E>>, Error<E>>
where
    F: Fn(&'a Group<D, E>) -> Fut,
    Fut: Future<Output = Result<(), Error<E>>>,
{
    let mut group: Option<&'a Group<D, E>> = None;

    while let Some(name) = segments.current() {
        if let Some(g) = conf.groups.get_by_name(&*name) {
            if let Some(group) = group {
                if !group.subgroups.contains(&g.id) {
                    break;
                }
            }

            f(g).await?;

            segments.next();
            group = Some(g);
            continue;
        }

        // No more groups to be found.
        break;
    }

    Ok(group)
}

// TODO: Revamp and finish the documentation!
/// Parses a [`Command`] out of the content.
///
/// The last command and the group it or its predecessor belongs to are returned.
///
/// A content can have an infinite amount of commands, so long they
/// are subcommands of the commands preceeding them.
///
/// This function behaves differently when a group is passed or not. If the
/// group is present, the command that will be returned may be:
/// 1. from the group's commands list; or
/// 2. a subcommand (or its subcommand...) of the command from the group; or
/// 3. the group's default command if defined; or
/// 4. a subcommand (or its subcommand...) of the default command.
///
/// If the group is not the present, only the 1. and 2. points count.
///
/// ## If the group is present
///
/// If the passed segments are empty, and the default command is not defined,
/// the [`MissingContent`] error is returned. If the latter is defined, it is
/// then chosen as the initial command.
///
/// ## If the group is not present
///
/// If the passed segments are empty, the [`MissingContent`] error is returned.
///
/// ## If the group is present
///
/// If the first segment is not a command name, and the default command is not
/// defined, the [`InvalidCommandName`] error is returned. If the latter is defined,
/// it is then chosen as the initial command.
///
/// ## If the group is not present
///
/// If the first segment is a name not affiliated with any command, the [`InvalidCommandName`]
/// error is returned. Otherwise, the command under that name is chosen as the initial command.
///
/// After the initial command is determined, it is examined if it is blocked.
/// In the case that it is blocked, the [`BlockedCommand`] error is returned.
///
/// If the group is not present, a search through [`Configuration::top_level_groups`]
/// will be conducted. If this fails, the [`InvalidCommand`] error without the group id
/// is returned.
///
/// As the function successfully parses a command, the given `f` async function
/// is called with the command and its group. Its purpose is to short-circuit
/// this function if the command does not conform to certain conditions,
/// such as its checks failing. If `f` returns an error, it is propagated
/// to the caller of this function.
///
/// [`Command`]: ../../command/struct.Command.html
/// [`MissingContent`]: ../../error/enum.DispatchError.html#variant.MissingContent
pub async fn command<'a, D, E, F, Fut>(
    conf: &'a Configuration<D, E>,
    segments: &mut Segments<'_>,
    group: Option<&'a Group<D, E>>,
    f: F,
) -> Result<(&'a Group<D, E>, &'a Command<D, E>), Error<E>>
where
    F: Fn(&'a Group<D, E>, &'a Command<D, E>) -> Fut,
    Fut: Future<Output = Result<(), Error<E>>>,
{
    let default_command = default_command(conf, group);
    let mut command = match initial_command(conf, segments) {
        Ok(cmd) => match default_command {
            // Handle the current command as the default command's subcommand later.
            Some(default) if default.subcommands.contains(&cmd.id) => default,
            _ => {
                segments.next();

                cmd
            }
        },
        Err(e) => default_command.ok_or(e)?,
    };

    let group = match group {
        Some(group) if group.commands.contains(&command.id) => group,
        Some(group) => {
            return Err(Error::Dispatch(DispatchError::InvalidCommand(
                Some(group.id),
                command.id,
            )))
        }
        None => match conf
            .top_level_groups
            .iter()
            .find(|g| g.commands.contains(&command.id))
        {
            Some(group) => group,
            None => {
                return Err(Error::Dispatch(DispatchError::InvalidCommand(
                    None, command.id,
                )))
            }
        },
    };

    f(group, command).await?;

    while let Some(name) = segments.current() {
        if let Some(cmd) = conf.commands.get_by_name(&*name) {
            if !command.subcommands.contains(&cmd.id) {
                break;
            }

            f(group, cmd).await?;

            segments.next();
            command = cmd;
            continue;
        }

        break;
    }

    Ok((group, command))
}

fn default_command<'a, D, E>(
    conf: &'a Configuration<D, E>,
    group: Option<&Group<D, E>>,
) -> Option<&'a Command<D, E>> {
    group
        .and_then(|g| g.default_command)
        .map(|id| &conf.commands[id])
}

fn initial_command<'a, D, E>(
    conf: &'a Configuration<D, E>,
    segments: &mut Segments<'_>,
) -> Result<&'a Command<D, E>, Error<E>> {
    match segments.current() {
        Some(name) => match conf.commands.get_by_name(&*name) {
            Some(cmd) => Ok(cmd),
            None => Err(Error::Dispatch(DispatchError::InvalidCommandName(
                name.into_owned(),
            ))),
        },
        None => Err(Error::Dispatch(DispatchError::MissingContent)),
    }
}
