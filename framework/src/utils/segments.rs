//! Functions and types for handling *segments*.
//!
//! A segment is a substring of a source string. The boundaries of the substring
//! are determined by a delimiter, which is a [`char`] value.

use std::borrow::Cow;

/// Returns the index to the end of a segment in the source.
///
/// If the delimiter could not be found in the source, the length of the source
/// is returned instead.
///
/// # Examples
///
/// ```rust
/// use serenity_framework::utils::segment_index;
///
/// assert_eq!(segment_index("hello world", ' '), 5);
/// assert_eq!(segment_index("world", ' '), "world".len());
/// ```
pub fn segment_index(src: &str, delimiter: char) -> usize {
    src.find(delimiter).unwrap_or_else(|| src.len())
}

/// Returns a segment of the source.
///
/// If the source is empty, `None` is returned.
///
/// # Examples
///
/// ```rust
/// use serenity_framework::utils::segment;
///
/// assert_eq!(segment("hello world", ' '), Some("hello"));
/// assert_eq!(segment("world", ' '), Some("world"));
/// assert_eq!(segment("", ' '), None);
/// ```
pub fn segment(src: &str, delimiter: char) -> Option<&str> {
    if src.is_empty() {
        None
    } else {
        Some(&src[..segment_index(src, delimiter)])
    }
}

/// Returns a segment and the rest of the source after the delimiter.
///
/// If the delimiter appears many times after the segment, all instances of it
/// are removed.
///
/// If the source is empty, `None` is returned.
///
/// # Examples
///
/// ```rust
/// use serenity_framework::utils::segment_split;
///
/// assert_eq!(segment_split("hello   world", ' '), Some(("hello", "world")));
/// assert_eq!(segment_split("world", ' '), Some(("world", "")));
/// assert_eq!(segment_split("", ' '), None);
/// ```
pub fn segment_split(src: &str, delimiter: char) -> Option<(&'_ str, &'_ str)> {
    if src.is_empty() {
        None
    } else {
        let (segment, rest) = src.split_at(segment_index(src, delimiter));
        Some((segment, rest.trim_start_matches(delimiter)))
    }
}

/// An iterator type that splits a string into segments using a delimiter.
///
/// It returns [`Cow`] values to handle case sensitivity.
///
/// [`Cow::Borrowed`] is returned if the [`case_insensitive`] field is `false`,
/// as the segment is a slice to the string.
///
/// [`Cow::Owned`] is returned if [`case_insensitive`] is `true`, as the segment
/// is converted to lowercase using [`str::to_lowercase`].
///
/// # Examples
///
/// ```rust
/// use serenity_framework::utils::Segments;
///
/// use std::borrow::Cow;
///
/// let mut iter = Segments::new("hello world", ' ', false);
///
/// assert_eq!(iter.next(), Some(Cow::Borrowed("hello")));
/// assert_eq!(iter.next(), Some(Cow::Borrowed("world")));
/// assert_eq!(iter.next(), None);
///
/// let mut iter = Segments::new("hElLo WOrLd", ' ', true);
///
/// assert_eq!(iter.next(), Some(Cow::Owned("hello".to_string())));
/// assert_eq!(iter.next(), Some(Cow::Owned("world".to_string())));
/// assert_eq!(iter.next(), None);
/// ```
///
/// [`Cow`]: std::borrow::Cow
/// [`case_insensitive`]: Segments::case_insensitive
#[derive(Debug, Clone)]
pub struct Segments<'a> {
    src: &'a str,
    delimiter: char,
    case_insensitive: bool,
}

impl<'a> Segments<'a> {
    /// Creates a `Segments` instance.
    pub fn new(src: &'a str, delimiter: char, case_insensitive: bool) -> Self {
        Self {
            src,
            delimiter,
            case_insensitive,
        }
    }

    /// Returns the source string from which segments are constructed.
    pub fn source(&self) -> &'a str {
        self.src
    }

    /// Returns the delimiter character that is used to determine the boundaries
    /// of a segment.
    pub fn delimiter(&self) -> char {
        self.delimiter
    }

    /// Returns the boolean that determines whether to ignore casing of segments.
    pub fn case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    /// Returns a boolean indicating that the source string is empty.
    pub fn is_empty(&self) -> bool {
        self.src.is_empty()
    }

    /// Returns the current segment.
    ///
    /// If the source string is empty, `None` is returned.
    pub fn current(&self) -> Option<Cow<'a, str>> {
        let segment = segment(self.src, self.delimiter)?;

        if self.case_insensitive {
            Some(Cow::Owned(segment.to_lowercase()))
        } else {
            Some(Cow::Borrowed(segment))
        }
    }
}

impl<'a> Iterator for Segments<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        let (segment, rest) = segment_split(self.src, self.delimiter)?;

        self.src = rest;

        Some(if self.case_insensitive {
            Cow::Owned(segment.to_lowercase())
        } else {
            Cow::Borrowed(segment)
        })
    }
}
