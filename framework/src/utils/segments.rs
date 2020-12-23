//! Functions and types for handling *segments*.
//!
//! A segment is a substring of a source string. The boundaries of the substring
//! are determined by a delimiter, which is a &[`str`] value.

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
/// assert_eq!(segment_index("hello world", " "), 5);
/// assert_eq!(segment_index("world", " "), "world".len());
/// ```
pub fn segment_index(src: &str, delimiter: &str) -> usize {
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
/// assert_eq!(segment("", " "), None);
/// assert_eq!(segment("hello world", " "), Some("hello"));
/// assert_eq!(segment("world", " "), Some("world"));
/// ```
pub fn segment<'a>(src: &'a str, delimiter: &str) -> Option<&'a str> {
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
/// assert_eq!(segment_split("hello   world", " "), Some(("hello", "world")));
/// assert_eq!(segment_split("world", " "), Some(("world", "")));
/// assert_eq!(segment_split("", " "), None);
/// ```
pub fn segment_split<'a>(src: &'a str, delimiter: &str) -> Option<(&'a str, &'a str)> {
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
/// let mut iter = Segments::new("hello world", " ", false);
///
/// assert_eq!(iter.next(), Some(Cow::Borrowed("hello")));
/// assert_eq!(iter.next(), Some(Cow::Borrowed("world")));
/// assert_eq!(iter.next(), None);
///
/// let mut iter = Segments::new("hElLo WOrLd", " ", true);
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
    delimiter: &'a str,
    case_insensitive: bool,
}

impl<'a> Segments<'a> {
    /// Creates a `Segments` instance.
    pub fn new(src: &'a str, delimiter: &'a str, case_insensitive: bool) -> Self {
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

    /// Sets the new source string from which segments are constructed.
    pub fn set_source(&mut self, src: &'a str) {
        self.src = src;
    }

    /// Returns the delimiter string that is used to determine the boundaries
    /// of a segment.
    pub fn delimiter(&self) -> &'a str {
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

/// Returns a quoted segment and the rest of the source.
///
/// A quoted segment is a part of the source that is encompassed by quotation marks.
/// Or, if a leading quotation mark exists, but the trailing mark is missing,
/// the quoted segment is the rest of the source excluding the leading mark.
///
/// If the source is empty or the source does not start with a leading quotation mark,
/// `None` is returned.
///
/// # Examples
///
/// ```
/// // Used example strings are from the YouTube video https://www.youtube.com/watch?v=1edPxKqiptw
/// use serenity_framework::utils::quoted_segment_split;
///
/// assert_eq!(quoted_segment_split(""), None);
/// assert_eq!(quoted_segment_split("Doll and roll"), None);
/// assert_eq!(quoted_segment_split("\"and some\" and home."), Some(("and some", " and home.")));
/// assert_eq!(quoted_segment_split("\"Stranger does not rhyme with anger"), Some(("Stranger does not rhyme with anger", "")));
/// ```
pub fn quoted_segment_split(src: &str) -> Option<(&str, &str)> {
    if src.is_empty() || !src.starts_with('"') {
        return None;
    }

    let src = &src[1..];

    match src.find('"') {
        Some(index) => Some((&src[..index], &src[(index + 1)..])),
        None => Some((src, "")),
    }
}

/// Returns a quoted segment of the source.
///
/// Refer to [`quoted_segment_split`] for the definition of a quoted segment.
///
/// If the source is empty or the source does not start with a leading quotation mark,
/// `None` is returned.
///
/// # Examples
///
/// ```
/// // Used example strings are from the YouTube video https://www.youtube.com/watch?v=1edPxKqiptw
/// use serenity_framework::utils::quoted_segment;
///
/// assert_eq!(quoted_segment(""), None);
/// assert_eq!(quoted_segment("Neither does devour with clangour"), None);
/// assert_eq!(quoted_segment("\"Souls but\" foul"), Some("Souls but"));
/// assert_eq!(quoted_segment("\"haunt but aunt"), Some("haunt but aunt"));
/// ```
pub fn quoted_segment(src: &str) -> Option<&str> {
    quoted_segment_split(src).map(|(seg, _)| seg)
}

/// Returns an argument segment and the rest of the source.
///
/// An argument segment is either [a quoted segment][qseg]
/// or [a normal segment][seg].
///
/// When the segment is quoted, the rest of the source is trimmed off of
/// the specified `delimiter`.
///
/// If the source is empty, `None` is returned.
///
/// # Examples
///
/// ```
/// // Used example strings are from the YouTube video https://www.youtube.com/watch?v=1edPxKqiptw
/// use serenity_framework::utils::argument_segment_split;
///
/// assert_eq!(argument_segment_split("", ", "), None);
/// assert_eq!(argument_segment_split("Font, front, wont", ", "), Some(("Font", "front, wont")));
/// assert_eq!(argument_segment_split("\"want, grand\", and grant", ", "), Some(("want, grand", "and grant")));
/// assert_eq!(argument_segment_split("\"Shoes, goes, does.", ", "), Some(("Shoes, goes, does.", "")));
/// ```
///
/// [qseg]: quoted_segment_split
/// [seg]: segment
pub fn argument_segment_split<'a>(src: &'a str, delimiter: &str) -> Option<(&'a str, &'a str)> {
    match quoted_segment_split(src) {
        Some((segment, rest)) => Some((segment, rest.trim_start_matches(delimiter))),
        None => segment_split(src, delimiter),
    }
}

/// Returns an argument segment of the source.
///
/// Refer to [`argument_segment_split`] for the definition of an argument segment.
///
/// If the source is empty, `None` is returned.
///
/// # Examples
///
/// ```
/// // Used example strings are from the YouTube video https://www.youtube.com/watch?v=1edPxKqiptw
/// use serenity_framework::utils::argument_segment;
///
/// assert_eq!(argument_segment("", ", "), None);
/// assert_eq!(argument_segment("Now first say finger, ", ", "), Some("Now first say finger"));
/// assert_eq!(argument_segment("\"And then singer, ginger\", linger, ", ", "), Some("And then singer, ginger"));
/// assert_eq!(argument_segment("\"Real, zeal, mauve", ", "), Some("Real, zeal, mauve"));
/// ```

pub fn argument_segment<'a>(src: &'a str, delimiter: &str) -> Option<&'a str> {
    argument_segment_split(src, delimiter).map(|(seg, _)| seg)
}

/// An iterator type that splits a string into [argument segments][aseg] using a delimiter and quotes.
///
/// # Examples
///
/// ```rust
/// // Used example strings are from the YouTube video https://www.youtube.com/watch?v=1edPxKqiptw
/// use serenity_framework::utils::ArgumentSegments;
///
/// let mut iter = ArgumentSegments::new("Marriage, \"foliage, mirage\", \"and age.", ", ");
///
/// assert_eq!(iter.next(), Some("Marriage"));
/// assert_eq!(iter.next(), Some("foliage, mirage"));
/// assert_eq!(iter.next(), Some("and age."));
/// assert_eq!(iter.next(), None);
/// ```
///
/// [aseg]: argument_segment_split
#[derive(Debug, Clone)]
pub struct ArgumentSegments<'a> {
    src: &'a str,
    delimiter: &'a str,
}

impl<'a> ArgumentSegments<'a> {
    /// Creates a new `ArgumentSegments` instance.
    pub fn new(src: &'a str, delimiter: &'a str) -> Self {
        Self {
            src,
            delimiter
        }
    }

    /// Returns the source string from which segments are constructed.
    pub fn source(&self) -> &'a str {
        self.src
    }

    /// Sets the new source string from which segments are constructed.
    pub fn set_source(&mut self, src: &'a str) {
        self.src = src;
    }

    /// Returns the delimiter string that is used to determine the boundaries
    /// of a segment.
    pub fn delimiter(&self) -> &'a str {
        self.delimiter
    }

    /// Returns a boolean indicating that the source string is empty.
    pub fn is_empty(&self) -> bool {
        self.src.is_empty()
    }
}

impl<'a> Iterator for ArgumentSegments<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let (segment, rest) = argument_segment_split(self.src, self.delimiter)?;

        self.src = rest;

        Some(segment)
    }
}
