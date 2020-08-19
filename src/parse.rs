use crate::command::{Command, CommandMap};
use crate::group::{Group, GroupMap};
use crate::Configuration;

use std::iter::Peekable;

pub fn prefix<'a, D, E>(conf: &Configuration<D, E>, content: &'a str) -> Option<&'a str> {
    if content.starts_with(&conf.prefix) {
        Some(&content[conf.prefix.len()..])
    } else {
        None
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

pub fn segments(src: &str, delimiter: char) -> Peekable<Segments<'_>> {
    Segments { src, delimiter }.peekable()
}

#[derive(Debug)]
pub struct Groups<'a, 'b, D, E> {
    map: &'a GroupMap<D, E>,
    segments: &'b mut Peekable<Segments<'a>>,
}

impl<'a, 'b, D, E> Iterator for Groups<'a, 'b, D, E> {
    type Item = &'a Group<D, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.segments.peek()?;
        let group = self.map.get_by_name(*name)?;

        self.segments.next();
        self.map = &group.subgroups;

        Some(group)
    }
}

pub fn groups<'a, 'b, D, E>(
    map: &'a GroupMap<D, E>,
    segments: &'b mut Peekable<Segments<'a>>,
) -> Groups<'a, 'b, D, E> {
    Groups { map, segments }
}

#[derive(Debug)]
pub struct Commands<'a, 'b, D, E> {
    map: &'a CommandMap<D, E>,
    segments: &'b mut Peekable<Segments<'a>>,
}

impl<'a, 'b, D, E> Iterator for Commands<'a, 'b, D, E> {
    type Item = &'a Command<D, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.segments.peek()?;
        let command = self.map.get_by_name(*name)?;

        self.segments.next();
        self.map = &command.subcommands;

        Some(command)
    }
}

pub fn commands<'a, 'b, D, E>(
    map: &'a CommandMap<D, E>,
    segments: &'b mut Peekable<Segments<'a>>,
) -> Commands<'a, 'b, D, E> {
    Commands { map, segments }
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
