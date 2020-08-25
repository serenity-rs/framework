use crate::context::PrefixContext;

use serenity::model::channel::Message;

/// Parses a mention from the message. A mention is defined as text starting with `<@`,
/// which may be followed by `!`, proceeded by a user id, and ended by a `>`.
///
/// This can be expressed in a regular expression as `<@!?\d+>`.
///
/// As an example, these are valid mentions:
/// - <@110372470472613888>
/// - <@!110372470472613888>
///
/// Returns the mention and the [`content`].
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

/// Parses a prefix from the message. A prefix is defined as
///
/// 1: a [mention]
/// 2: a [dynamically chosen prefix][dyn_prefix]
/// 3: or a [statically defined prefix from a list][prefixes],
///
/// It is parsed in that order.
///
/// The prefix and the [`content`] are returned on success.
///
/// [mention]: crate::configuration::Configuration::on_mention
/// [dyn_prefix]: crate::configuration::Configuration::dynamic_prefix
/// [prefixes]: crate::configuration::Configuration::prefixes
pub async fn prefix<'a, D, E>(
    ctx: PrefixContext<'_, D, E>,
    msg: &'a Message,
) -> Option<(&'a str, &'a str)> {
    if let Some(id) = &ctx.conf.on_mention {
        if let Some(pair) = mention(&msg.content, &id) {
            return Some(pair);
        }
    }

    if let Some(dynamic_prefix) = ctx.conf.dynamic_prefix {
        if let Some(index) = dynamic_prefix(&ctx, msg).await {
            return Some(msg.content.split_at(index));
        }
    }

    if let Some(prefix) = ctx
        .conf
        .prefixes
        .iter()
        .find(|p| msg.content.starts_with(p.as_str()))
    {
        Some(msg.content.split_at(prefix.len()))
    } else {
        None
    }
}

/// Parses the content of the message. The content is defined as the substring of
/// the message after the prefix. If the [`Configuration::no_dm_prefix`] option is enabled,
/// the content is the whole message.
///
/// The prefix and the content are returned on success.
///
/// [`Configuration::no_dm_prefix`]: crate::configuration::Configuration::no_dm_prefix
pub async fn content<'a, D, E>(
    ctx: PrefixContext<'_, D, E>,
    msg: &'a Message,
) -> Option<(&'a str, &'a str)> {
    if msg.is_private() && ctx.conf.no_dm_prefix {
        Some(("", &msg.content))
    } else {
        prefix(ctx, msg).await
    }
}
