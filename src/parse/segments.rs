use std::borrow::Cow;

/// An [`Iterator`] type that splits a string into *segments* by a delimiter.
/// It returns [`Cow`] values to handle case sensitivity. [`Cow::Borrowed`] is
/// returned if the [`case_insensitive`] field is `false`, as the segment is a
/// slice to the string. [`Cow::Owned`] is returned if [`case_insensitive`] is
/// `true`, as the segment is converted to lowercase using [`str::to_lowercase`].
///
/// [`Iterator`]: std::iter::Iterator
/// [`Cow`]: std::borrow::Cow
/// [`Cow::Borrowed`]: std::borrow::Cow::Borrowed
/// [`Cow::Owned`]: std::borrow::Cow::Owned
/// [`case_insensitive`]: Segments::case_insensitive
/// [`str::to_lowercase`]: str::to_lowercase
#[derive(Debug, Clone)]
pub struct Segments<'a> {
    pub src: &'a str,
    pub delimiter: char,
    pub case_insensitive: bool,
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
    use super::*;

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
