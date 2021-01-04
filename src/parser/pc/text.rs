/// Deals with characters and strings.
/// The Reader is always a Reader<Item = char>
use crate::parser::pc::{is_eol_or_whitespace, is_whitespace, Parser, Reader, ReaderResult, Undo};
use std::marker::PhantomData;

/// A parser that finds a specific string, case insensitive.
pub struct StringParser<R: Reader<Item = char>> {
    needle: &'static str,
    reader: PhantomData<R>,
}

/// Parses the given string, case insensitive.
pub fn string_p<R: Reader<Item = char>>(needle: &'static str) -> StringParser<R> {
    StringParser {
        needle,
        reader: PhantomData,
    }
}

impl<R> Parser<R> for StringParser<R>
where
    R: Reader<Item = char>,
{
    type Output = String;
    fn parse(&self, r: R) -> ReaderResult<R, String, R::Err> {
        let mut reader = r;
        let mut result = String::new();
        for n in self.needle.chars() {
            let res = reader.read();
            match res {
                Ok((r, Some(ch))) => {
                    result.push(ch);
                    if ch.to_ascii_uppercase() == n.to_ascii_uppercase() {
                        reader = r;
                    } else {
                        return Ok((r.undo(result), None));
                    }
                }
                Ok((r, None)) => {
                    // EOF before matching all characters, undo collected and return None
                    return Ok((r.undo(result), None));
                }
                Err((r, err)) => {
                    // Error occurred, exit fast
                    return Err((r, err));
                }
            }
        }
        Ok((reader, Some(result)))
    }
}

/// Defines a character sequence with a leading character set and
/// subsequent characters.
///
/// A parser is automatically implemented for implementations of this trait.
pub trait CharSequence {
    /// Checks if the given character can be the first in the sequence.
    fn is_leading(ch: char) -> bool {
        Self::is_valid(ch)
    }

    /// Checks if the given character can belong in the sequence.
    fn is_valid(ch: char) -> bool;
}

impl<R, S: CharSequence> Parser<R> for S
where
    R: Reader<Item = char>,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let mut r = reader;
        let (tmp, opt_first) = r.read()?;
        r = tmp;
        match opt_first {
            Some(first) => {
                if !S::is_leading(first) {
                    return Ok((r.undo_item(first), None));
                }
                let mut buf = String::new();
                buf.push(first);
                loop {
                    let (tmp, opt_next) = r.read()?;
                    r = tmp;
                    match opt_next {
                        Some(next) => {
                            if S::is_valid(next) {
                                buf.push(next);
                            } else {
                                r = r.undo_item(next);
                                break;
                            }
                        }
                        _ => {
                            break;
                        }
                    }
                }
                Ok((r, Some(buf)))
            }
            _ => Ok((r, None)),
        }
    }
}

#[macro_export]
macro_rules! char_sequence_p {
    ($struct_name:tt, $fn_name:tt, $is_valid:expr) => {
        pub fn $fn_name<R: crate::parser::pc::Reader<Item = char>>(
        ) -> impl crate::parser::pc::Parser<R, Output = String> {
            $struct_name {}
        }

        struct $struct_name {}

        impl crate::parser::pc::text::CharSequence for $struct_name {
            fn is_valid(ch: char) -> bool {
                $is_valid(ch)
            }
        }
    };

    ($struct_name:tt, $fn_name:tt, $is_leading:expr, $is_valid:expr) => {
        pub fn $fn_name<R: crate::parser::pc::Reader<Item = char>>(
        ) -> impl crate::parser::pc::Parser<R, Output = String> {
            $struct_name {}
        }

        pub struct $struct_name {}

        impl crate::parser::pc::text::CharSequence for $struct_name {
            fn is_leading(ch: char) -> bool {
                $is_leading(ch)
            }

            fn is_valid(ch: char) -> bool {
                $is_valid(ch)
            }
        }
    };
}

// Parses one or more whitespace characters.
char_sequence_p!(Whitespace, whitespace_p, is_whitespace);

pub struct OptWhitespace {
    reject_empty: bool,
}

impl<R> Parser<R> for OptWhitespace
where
    R: Reader<Item = char>,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let mut buf = String::new();
        let mut r = reader;
        loop {
            let (tmp, opt_item) = r.read()?;
            r = tmp;
            match opt_item {
                Some(' ') => {
                    buf.push(' ');
                }
                Some(item) => {
                    r = r.undo(item);
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        if self.reject_empty && buf.is_empty() {
            Ok((r, None))
        } else {
            Ok((r, Some(buf)))
        }
    }
}

pub fn opt_whitespace_p<R>(reject_empty: bool) -> impl Parser<R, Output = String>
where
    R: Reader<Item = char>,
{
    OptWhitespace { reject_empty }
}

// Parses one or more characters consisting of whitespace and/or eol.
char_sequence_p!(EolOrWhitespace, eol_or_whitespace_p, is_eol_or_whitespace);

pub struct FollowedByOptWhitespace<S>(S);

impl<R, S> Parser<R> for FollowedByOptWhitespace<S>
where
    R: Reader<Item = char>,
    S: Parser<R>,
{
    type Output = S::Output;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => {
                let (reader, _) = opt_whitespace_p(false).parse(reader)?;
                Ok((reader, Some(item)))
            }
            _ => Ok((reader, None)),
        }
    }
}

pub struct PrecededByOptWhitespace<S>(S);

impl<R, S> Parser<R> for PrecededByOptWhitespace<S>
where
    R: Reader<Item = char>,
    S: Parser<R>,
{
    type Output = S::Output;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let (reader, opt_whitespace) = opt_whitespace_p(false).parse(reader)?;
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => Ok((reader, Some(item))),
            _ => Ok((reader.undo(opt_whitespace.unwrap_or_default()), None)),
        }
    }
}

pub struct SurroundedByOptWhitespace<S>(S);

impl<R, S> Parser<R> for SurroundedByOptWhitespace<S>
where
    R: Reader<Item = char>,
    S: Parser<R>,
{
    type Output = S::Output;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let (reader, opt_leading_whitespace) = opt_whitespace_p(false).parse(reader)?;
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => {
                let (reader, _) = opt_whitespace_p(false).parse(reader)?;
                Ok((reader, Some(item)))
            }
            _ => Ok((
                reader.undo(opt_leading_whitespace.unwrap_or_default()),
                None,
            )),
        }
    }
}

/// Offers chaining methods for parsers where the reader's item is `char`.
pub trait TextParser<R: Reader<Item = char>>: Parser<R> + Sized {
    /// Allows for optional whitespace after this parser's successful result.
    fn followed_by_opt_ws(self) -> FollowedByOptWhitespace<Self> {
        FollowedByOptWhitespace(self)
    }

    /// Allows for optional whitespace before this parser's successful result.
    /// If the parser fails, the whitespace will be undone.
    fn preceded_by_opt_ws(self) -> PrecededByOptWhitespace<Self> {
        PrecededByOptWhitespace(self)
    }

    /// Allows for optional whitespace around this parser's successful result.
    /// If the parser fails, the leading whitespace will be undone.
    fn surrounded_by_opt_ws(self) -> SurroundedByOptWhitespace<Self> {
        SurroundedByOptWhitespace(self)
    }
}

impl<R: Reader<Item = char>, T> TextParser<R> for T where T: Parser<R> {}
