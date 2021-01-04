use std::marker::PhantomData;

use crate::parser::pc::binary::{BinaryParser, LeftAndOptRight, OptLeftAndRight};
use crate::parser::pc::unary::PeekReaderItem;
use crate::parser::pc::{is_eol_or_whitespace, is_whitespace, Item, Reader, ReaderResult, Undo};
use crate::parser::pc_specific::{
    is_digit, is_letter, is_non_leading_identifier_with_dot, is_non_leading_identifier_without_dot,
};

/// Deals with characters and strings.
/// The Reader is always a Reader<Item = char>
use super::Parser;

/// A parser that finds a specific string.
pub struct StringRecognizer<R: Reader<Item = char>> {
    needle: &'static str,
    reader: PhantomData<R>,
}

/// Parses the given string.
pub fn string_p<R: Reader<Item = char>>(needle: &'static str) -> StringRecognizer<R> {
    StringRecognizer {
        needle,
        reader: PhantomData,
    }
}

impl<R> Parser<R> for StringRecognizer<R>
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

/// Read one or more characters that meet the predicate
pub struct StringWhile<R, F>(PhantomData<R>, F, bool);

impl<R, F> StringWhile<R, F>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    pub fn new(predicate: F, reject_empty: bool) -> Self {
        Self(PhantomData, predicate, reject_empty)
    }
}

impl<R, F> Parser<R> for StringWhile<R, F>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    type Output = String;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        do_string_while(reader, &self.1, self.2)
    }
}

fn do_string_while<R, F>(
    reader: R,
    predicate: F,
    reject_empty: bool,
) -> ReaderResult<R, String, R::Err>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    let mut result = String::new();
    let mut r = reader;
    let mut has_more = true;
    while has_more {
        let (tmp, opt_item) = r.read()?;
        r = tmp;
        match opt_item {
            Some(item) => {
                if predicate(item) {
                    result.push(item);
                } else {
                    r = r.undo_item(item);
                    has_more = false;
                }
            }
            None => {
                has_more = false;
            }
        }
    }
    if result.is_empty() && reject_empty {
        Ok((r, None))
    } else {
        Ok((r, Some(result)))
    }
}

pub fn string_while_p<R, F>(predicate: F) -> StringWhile<R, F>
where
    R: Reader<Item = char>,
    F: Fn(char) -> bool,
{
    StringWhile::new(predicate, true)
}

macro_rules! recognize_while_predicate {
    ($struct_name:tt, $fn_name:tt, $predicate:expr) => {
        pub struct $struct_name<R: Reader<Item = char>>(PhantomData<R>, bool);

        impl<R: Reader<Item = char>> $struct_name<R> {
            pub fn new(reject_empty: bool) -> Self {
                Self(PhantomData, reject_empty)
            }
        }

        impl<R: Reader<Item = char>> Parser<R> for $struct_name<R> {
            type Output = String;

            fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
                do_string_while(reader, $predicate, self.1)
            }
        }

        pub fn $fn_name<R: Reader<Item = char>>() -> $struct_name<R> {
            $struct_name::new(true)
        }
    };
}

// Reads one or more whitespace
recognize_while_predicate!(Whitespace, whitespace_p, is_whitespace);
recognize_while_predicate!(Letters, letters_p, is_letter);
recognize_while_predicate!(
    LettersOrDigits,
    letters_or_digits_p,
    is_non_leading_identifier_without_dot
);
recognize_while_predicate!(
    LettersOrDigitsOrDots,
    letters_or_digits_or_dots_p,
    is_non_leading_identifier_with_dot
);
recognize_while_predicate!(Digits, digits_p, is_digit);
recognize_while_predicate!(EolOrWhitespace, eol_or_whitespace_p, is_eol_or_whitespace);

/// Converts the result of the underlying parser into a string.
pub struct Stringify<A>(A);

impl<A> Stringify<A> {
    pub fn new(source: A) -> Self {
        Self(source)
    }
}

impl<R, X, Y> Parser<R> for Stringify<LeftAndOptRight<X, Y>>
where
    R: Reader,
    X: Parser<R>,
    Y: Parser<R>,
    X::Output: std::fmt::Display,
    Y::Output: std::fmt::Display,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some((left, Some(right))) => Ok((reader, Some(format!("{}{}", left, right)))),
            Some((left, None)) => Ok((reader, Some(left.to_string()))),
            _ => Ok((reader, None)),
        }
    }
}

impl<R, X, Y> Parser<R> for Stringify<OptLeftAndRight<X, Y>>
where
    R: Reader + Undo<X::Output>,
    X: Parser<R>,
    Y: Parser<R>,
    X::Output: std::fmt::Display,
    Y::Output: std::fmt::Display,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some((Some(left), right)) => Ok((reader, Some(format!("{}{}", left, right)))),
            Some((None, right)) => Ok((reader, Some(right.to_string()))),
            _ => Ok((reader, None)),
        }
    }
}

impl<R> Parser<R> for Stringify<PeekReaderItem<Item<R>>>
where
    R: Reader<Item = char>,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => {
                let mut s: String = String::new();
                s.push(item);
                Ok((reader, Some(s)))
            }
            _ => Ok((reader, None)),
        }
    }
}

/// Offers chaining methods for parsers where the reader's item is `char`.
pub trait TextParser<R: Reader<Item = char>>: Parser<R> + Sized {
    /// Converts the result of this parser into a string.
    fn stringify(self) -> Stringify<Self> {
        Stringify::new(self)
    }

    /// Allows for optional whitespace after this parser's successful result.
    fn followed_by_opt_ws(self) -> LeftAndOptRight<Self, Whitespace<R>> {
        self.and_opt(whitespace_p())
    }

    /// Allows for optional whitespace before this parser's successful result.
    /// If the parser fails, the whitespace will be undone.
    fn preceded_by_opt_ws(self) -> OptLeftAndRight<Whitespace<R>, Self> {
        self.preceded_by(whitespace_p())
    }

    /// Allows for optional whitespace around this parser's successful result.
    /// If the parser fails, the leading whitespace will be undone.
    fn surrounded_by_opt_ws(
        self,
    ) -> OptLeftAndRight<Whitespace<R>, LeftAndOptRight<Self, Whitespace<R>>> {
        self.followed_by_opt_ws().preceded_by_opt_ws()
    }
}

impl<R: Reader<Item = char>, T> TextParser<R> for T where T: Parser<R> {}
