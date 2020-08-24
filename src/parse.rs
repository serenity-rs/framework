use crate::context::PrefixContext;

use serenity::model::channel::Message;

use std::borrow::Cow;

pub fn mention<'a>(content: &'a str, id: &str) -> Option<(&'a str, &'a str)> {
    if !content.starts_with("<@") {
        return None;
    }

    let content = content[2..].trim_start_matches('!');

    let index = content.find('>').unwrap_or(0);
    let mention = &content[..index];

    if mention == id {
        // + 1 to remove the angle bracket
        let (mention, mut rest) = content.split_at(index + 1);
        rest = rest.trim_start();
        Some((mention, rest))
    } else {
        None
    }
}

pub async fn prefix<'a, D, E>(
    ctx: PrefixContext<'a, D, E>,
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

pub async fn content<'a, D, E>(
    ctx: PrefixContext<'a, D, E>,
    msg: &'a Message,
) -> Option<(&'a str, &'a str)> {
    if msg.is_private() && ctx.conf.no_dm_prefix {
        Some(("", &msg.content))
    } else {
        prefix(ctx, msg).await
    }
}

#[derive(Debug, Clone)]
pub struct Segments<'a> {
    pub src: &'a str,
    pub delimiter: char,
    pub case_insensitive: bool,
}

impl<'a> Segments<'a> {
    pub fn new(src: &'a str, delimiter: char, case_insensitive: bool) -> Self {
        Self {
            src,
            delimiter,
            case_insensitive,
        }
    }
}

impl<'a> Iterator for Segments<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.src.is_empty() {
            return None;
        }

        let index = self
            .src
            .find(self.delimiter)
            .unwrap_or_else(|| self.src.len());
        let (segment, rest) = self.src.split_at(index);

        self.src = rest.trim_start_matches(self.delimiter);

        Some(if self.case_insensitive {
            Cow::Owned(segment.to_lowercase())
        } else {
            Cow::Borrowed(segment)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Segments;

    use std::borrow::Cow;

    #[test]
    fn segment_splitting() {
        let content = "Abc fOo      bar";
        let mut segments = Segments::new(content, ' ', false);

        assert_eq!(segments.next(), Some(Cow::Borrowed("Abc")));
        assert_eq!(segments.next(), Some(Cow::Borrowed("fOo")));
        assert_eq!(segments.next(), Some(Cow::Borrowed("bar")));
        assert_eq!(segments.next(), None);

        segments = Segments::new(content, ' ', true);

        assert_eq!(segments.next(), Some(Cow::Owned("abc".to_string())));
        assert_eq!(segments.next(), Some(Cow::Owned("foo".to_string())));
        assert_eq!(segments.next(), Some(Cow::Owned("bar".to_string())));
        assert_eq!(segments.next(), None);
    }
}
