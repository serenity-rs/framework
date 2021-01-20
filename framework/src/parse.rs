//! Utilities to parse the prefix and command out of a message.
//!
//! Refer to the [`content`] function for the definition of a prefix.

use crate::command::Command;
use crate::configuration::Configuration;
use crate::context::PrefixContext;
use crate::error::DispatchError;
use crate::utils::Segments;

use serenity::client::Context as SerenityContext;
use serenity::model::channel::Message;
use serenity::prelude::RwLock;

use std::sync::Arc;

/// Parses a mention from the message.
///
/// A mention is defined as text starting with `<@`, which may be followed by `!`,
/// proceeded by a user id, and ended by a `>`.
///
/// This can be expressed in a regular expression as `<@!?\d+>`.
///
/// As an example, these are valid mentions:
/// - <@110372470472613888>
/// - <@!110372470472613888>
///
/// Returns the mention and the rest of the message after the mention, with trimmed
/// whitespace.
pub fn mention<'a>(msg: &'a str, id: &str) -> Option<(&'a str, &'a str)> {
    if !msg.starts_with("<@") {
        return None;
    }

    let msg = msg[2..].trim_start_matches('!');

    let index = msg.find('>').unwrap_or(0);
    let mention = &msg[..index];

    if mention == id {
        // + 1 to remove the angle bracket
        let (mention, mut rest) = msg.split_at(index + 1);
        rest = rest.trim_start();
        Some((mention, rest))
    } else {
        None
    }
}

/// Parses a prefix from the message dynamically using the [`Configuration::dynamic_prefix`]
/// hook.
///
/// If the hook is not registered, or the hook returned `None`, `None` is returned.
/// Otherwise, the prefix and the rest of the message after the prefix is returned.
///
/// [`Configuration::dynamic_prefix`]: crate::configuration::Configuration::dynamic_prefix
#[allow(clippy::needless_lifetimes)]
pub async fn dynamic_prefix<'a, D, E>(
    ctx: PrefixContext<'_, D, E>,
    msg: &'a Message,
) -> Option<(&'a str, &'a str)> {
    if let Some(dynamic_prefix) = ctx.conf.dynamic_prefix {
        let index = dynamic_prefix(ctx, msg).await?;
        Some(msg.content.split_at(index))
    } else {
        None
    }
}

/// Parses a prefix from the message statically from a list of prefixes.
///
/// If none of the prefixes stored in the list are found in the message, `None` is returned.
/// Otherwise, the prefix and the rest of the message after the prefix is returned.
pub fn static_prefix<'a>(msg: &'a str, prefixes: &[String]) -> Option<(&'a str, &'a str)> {
    prefixes
        .iter()
        .find(|p| msg.starts_with(p.as_str()))
        .map(|p| msg.split_at(p.len()))
}

/// Returns the content of the message after parsing a prefix.
///
/// The content is defined as the substring of the message after the prefix.
/// If the [`Configuration::no_dm_prefix`] option is enabled, the content is
/// the whole message.
///
/// The prefix is defined as:
/// 1. a [mention]
/// 2. a [statically defined prefix from a list][prefixes]
/// 3. or a [dynamically chosen prefix][dyn_prefix]
///
/// It is parsed in that order.
///
/// If [`Configuration::no_dm_prefix`] is `false` and no prefix is found,
/// `None` is returned. Otherwise, the prefix and the content are returned.
///
/// [`Configuration::no_dm_prefix`]: crate::configuration::Configuration::no_dm_prefix
/// [prefixes]: static_prefix
/// [dyn_prefix]: dynamic_prefix
#[allow(clippy::needless_lifetimes)]
pub async fn content<'a, D, E>(
    data: &Arc<RwLock<D>>,
    conf: &Configuration<D, E>,
    serenity_ctx: &SerenityContext,
    msg: &'a Message,
) -> Option<(&'a str, &'a str)> {
    if msg.is_private() && conf.no_dm_prefix {
        return Some(("", &msg.content));
    }

    if let Some(on_mention) = &conf.on_mention {
        if let Some(pair) = mention(&msg.content, &on_mention) {
            return Some(pair);
        }
    }

    if let Some(pair) = static_prefix(&msg.content, &conf.prefixes) {
        return Some(pair);
    }

    let ctx = PrefixContext {
        data,
        conf,
        serenity_ctx,
    };

    dynamic_prefix(ctx, msg).await
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
    command: Option<&'a Command<D, E>>,
}

impl<'a, 'b, 'c, D, E> Iterator for CommandIterator<'a, 'b, 'c, D, E> {
    type Item = Result<&'a Command<D, E>, DispatchError>;

    fn next(&mut self) -> Option<Self::Item> {
        let checkpoint = self.segments.source();
        let name = self.segments.next()?;

        let cmd = match self.conf.commands.get_by_name(&*name) {
            Some(cmd) => cmd,
            None => {
                self.segments.set_source(checkpoint);

                // At least one valid command must be present in the message.
                // After the first command, we do not care if the "name" is invalid,
                // as it may be the argument to the command at that point.
                if self.command.is_none() {
                    return Some(Err(DispatchError::InvalidCommandName(name.into_owned())));
                }

                return None;
            },
        };

        if self.command.is_none() && !self.conf.root_level_commands.contains(&cmd.id) {
            self.segments.set_source(checkpoint);
            return None;
        }

        if let Some(command) = self.command {
            if !command.subcommands.contains(&cmd.id) {
                // We received a command, but it's not a subcommand of the previously
                // parsed command. Interpret it as an argument instead.
                //
                // This enables user-defined `help` commands.
                self.segments.set_source(checkpoint);
                return None;
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
/// The `Result` signifies whether a given name for the first command exists.
/// If it is not the case, the [`InvalidCommandName`] error is returned.
///
/// The `Option` returned from calling [`Iterator::next`] will signify whether the content had a
/// command, did not have a command, or was empty.
///
/// [iter]: self::CommandIterator
/// [`Command`]: crate::command::Command
/// [`DispatchError`]: crate::error::DispatchError
/// [`InvalidCommandName`]: crate::error::DispatchError::InvalidCommandName
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
