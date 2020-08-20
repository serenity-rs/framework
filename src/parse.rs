use crate::command::{Command, CommandMap};
use crate::group::{Group, GroupMap};
use crate::Configuration;

use serenity::model::channel::Message;

use std::iter::Peekable;

pub fn prefix<'a, D, E>(conf: &Configuration<D, E>, content: &'a str) -> Option<&'a str> {
    if let Some(mention) = &conf.on_mention {
        if content.starts_with("<@") {
            let content = content[2..].trim_start_matches('!');

            if let Some(index) = content.find('>') {
                let id = &content[..index];

                if id == mention {
                    // + 1 to remove the angle bracket
                    return Some(&content[index + 1..].trim_start());
                }
            }
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

pub struct Groups<'a, 'b, D, E, It>
where
    It: Iterator,
    It::Item: AsRef<str>,
{
    map: &'a GroupMap<D, E>,
    iter: &'b mut Peekable<It>,
}

impl<'a, 'b, D, E, It> Iterator for Groups<'a, 'b, D, E, It>
where
    It: Iterator,
    It::Item: AsRef<str>,
{
    type Item = &'a Group<D, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.iter.peek()?;
        let group = self.map.get_by_name(name.as_ref())?;

        self.iter.next();
        self.map = &group.subgroups;

        Some(group)
    }
}

pub fn groups<'a, 'b, D, E, It>(
    map: &'a GroupMap<D, E>,
    iter: &'b mut Peekable<It>,
) -> Groups<'a, 'b, D, E, It>
where
    It: Iterator,
    It::Item: AsRef<str>,
{
    Groups { map, iter }
}

pub struct Commands<'a, 'b, D, E, It>
where
    It: Iterator,
    It::Item: AsRef<str>,
{
    map: &'a CommandMap<D, E>,
    iter: &'b mut Peekable<It>,
}

impl<'a, 'b, D, E, It> Iterator for Commands<'a, 'b, D, E, It>
where
    It: Iterator,
    It::Item: AsRef<str>,
{
    type Item = &'a Command<D, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.iter.peek()?;
        let command = self.map.get_by_name(name.as_ref())?;

        self.iter.next();
        self.map = &command.subcommands;

        Some(command)
    }
}

pub fn commands<'a, 'b, D, E, It>(
    map: &'a CommandMap<D, E>,
    iter: &'b mut Peekable<It>,
) -> Commands<'a, 'b, D, E, It>
where
    It: Iterator,
    It::Item: AsRef<str>,
{
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
