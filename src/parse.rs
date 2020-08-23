use crate::Configuration;

use serenity::model::channel::Message;

use std::borrow::Cow;

pub fn mention<'a>(content: &'a str, id: &str) -> Option<&'a str> {
    if !content.starts_with("<@") {
        return None;
    }

    let content = content[2..].trim_start_matches('!');

    let index = content.find('>').unwrap_or(0);
    let mention = &content[..index];

    if mention == id {
        // + 1 to remove the angle bracket
        Some(&content[index + 1..].trim_start())
    } else {
        None
    }
}

pub fn prefix<'a, D, E>(conf: &Configuration<D, E>, content: &'a str) -> Option<&'a str> {
    if let Some(id) = &conf.on_mention {
        if let Some(content) = mention(content, &id) {
            return Some(content);
        }
    }

    if let Some(prefix) = conf
        .prefixes
        .iter()
        .find(|p| content.starts_with(p.as_str()))
    {
        Some(&content[prefix.len()..])
    } else {
        None
    }
}

pub fn content<'a, D, E>(conf: &Configuration<D, E>, msg: &'a Message) -> Option<&'a str> {
    if msg.is_private() && conf.no_dm_prefix {
        Some(&msg.content)
    } else {
        prefix(conf, &msg.content)
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

        let index = self.src.find(self.delimiter).unwrap_or(self.src.len());
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
        let content = "abc foo      bar";
        let mut segments = Segments::new(content, ' ', false);

        assert_eq!(segments.next(), Some(Cow::Borrowed("abc")));
        assert_eq!(segments.next(), Some(Cow::Borrowed("foo")));
        assert_eq!(segments.next(), Some(Cow::Borrowed("bar")));
        assert_eq!(segments.next(), None);
    }
}
