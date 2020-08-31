use crate::command::Command;
use crate::configuration::Configuration;
use crate::error::{DispatchError, Error};
use crate::group::Group;
use crate::utils::Segments;

use std::future::Future;

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

            if conf.blocked_entities.groups.contains(&g.id) {
                return Err(Error::Dispatch(DispatchError::BlockedGroup(g.id)));
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
    let default_command = default_command(conf, group.clone());
    let mut command = match segments.current() {
        Some(name) => match conf.commands.get_by_name(&*name) {
            Some(cmd) => match default_command {
                Some(default) if default.subcommands.contains(&cmd.id) => default,
                _ => {
                    segments.next();

                    cmd
                }
            },
            None => match default_command {
                Some(cmd) => cmd,
                None => {
                    return Err(Error::Dispatch(DispatchError::InvalidCommandName(
                        name.into_owned(),
                    )))
                }
            },
        },
        None => match default_command {
            Some(cmd) => cmd,
            None => return Err(Error::Dispatch(DispatchError::MissingContent)),
        },
    };

    if conf.blocked_entities.commands.contains(&command.id) {
        return Err(Error::Dispatch(DispatchError::BlockedCommand(command.id)));
    }

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

            if conf.blocked_entities.commands.contains(&cmd.id) {
                return Err(Error::Dispatch(DispatchError::BlockedCommand(cmd.id)));
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
