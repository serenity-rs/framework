use crate::command::{Command, CommandMap};
use crate::group::{Group, GroupMap};
use crate::Configuration;

use serenity::model::channel::Message;

use std::iter::Peekable;

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

    if let Some(prefix) = conf.prefixes.iter().find(|p| content.starts_with(p.as_str())) {
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
    src: &'a str,
    delimiter: char,
}

impl<'a> Iterator for Segments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.src.is_empty() {
            return None;
        }

        let index = self.src.find(self.delimiter).unwrap_or(self.src.len());
        let (segment, rest) = self.src.split_at(index);

        self.src = rest.trim_start_matches(self.delimiter);

        Some(segment)
    }
}

pub fn segments(src: &str, delimiter: char) -> Segments<'_> {
    Segments { src, delimiter }
}

pub struct Groups<'a, 'b, D, E> {
    map: &'a GroupMap<D, E>,
    iter: &'b mut Peekable<Segments<'a>>,
}

impl<'a, 'b, D, E> Iterator for Groups<'a, 'b, D, E> {
    type Item = &'a Group<D, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = *self.iter.peek()?;
        let group = self.map.get_by_name(name)?;

        self.iter.next();
        self.map = &group.subgroups;

        Some(group)
    }
}

pub fn groups<'a, 'b, D, E>(
    map: &'a GroupMap<D, E>,
    iter: &'b mut Peekable<Segments<'a>>,
) -> Groups<'a, 'b, D, E> {
    Groups { map, iter }
}

pub struct Commands<'a, 'b, D, E> {
    map: &'a CommandMap<D, E>,
    iter: &'b mut Peekable<Segments<'a>>,
}

impl<'a, 'b, D, E> Iterator for Commands<'a, 'b, D, E> {
    type Item = &'a Command<D, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = *self.iter.peek()?;
        let command = self.map.get_by_name(name)?;

        self.iter.next();
        self.map = &command.subcommands;

        Some(command)
    }
}

pub fn commands<'a, 'b, D, E>(
    map: &'a CommandMap<D, E>,
    iter: &'b mut Peekable<Segments<'a>>,
) -> Commands<'a, 'b, D, E> {
    Commands { map, iter }
}

#[cfg(test)]
mod tests {
    #[test]
    fn segment_splitting() {
        let content = "abc foo      bar";
        let mut segments = crate::parse::segments(content, ' ');

        assert_eq!(segments.next(), Some("abc"));
        assert_eq!(segments.next(), Some("foo"));
        assert_eq!(segments.next(), Some("bar"));
        assert_eq!(segments.next(), None);
    }
}
