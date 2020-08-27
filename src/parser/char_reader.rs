use crate::common::{CaseInsensitiveString, HasLocation, Location, QError};
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::pc::loc::*;
use crate::parser::pc::traits::*;
use crate::parser::types::{Keyword, Name, TypeQualifier};
use std::collections::VecDeque;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::str::FromStr;

fn is_letter(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')
}

fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

fn is_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\t'
}

fn is_eol(ch: char) -> bool {
    ch == '\r' || ch == '\n'
}

fn is_eol_or_whitespace(ch: char) -> bool {
    is_eol(ch) || is_whitespace(ch)
}

fn is_symbol(ch: char) -> bool {
    (ch > ' ' && ch < '0')
        || (ch > '9' && ch < 'A')
        || (ch > 'Z' && ch < 'a')
        || (ch > 'z' && ch <= '~')
}

pub trait ParserSource: Reader<Item = char, Err = QError> {}

impl IsNotFoundErr for QError {
    fn is_not_found_err(&self) -> bool {
        *self == QError::CannotParse
    }
}

impl NotFoundErr for QError {
    fn not_found_err() -> Self {
        QError::CannotParse
    }
}

impl<P: ParserSource> Undo<char> for P {
    fn undo(self, item: char) -> Self {
        self.undo_item(item)
    }
}

impl<P: ParserSource> Undo<()> for P {
    fn undo(self, _item: ()) -> Self {
        self
    }
}

impl<T: BufRead + 'static> Undo<Name> for EolReader<T> {
    fn undo(self, n: Name) -> Self {
        match n {
            Name::Bare(b) => self.undo(b),
            Name::Qualified { name, qualifier } => {
                let first = self.undo(qualifier);
                first.undo(name)
            }
        }
    }
}

impl<T: BufRead + 'static> Undo<TypeQualifier> for EolReader<T> {
    fn undo(self, s: TypeQualifier) -> Self {
        let ch: char = s.try_into().unwrap();
        self.undo(ch)
    }
}

/// Reads one character at a time out of a `BufRead`.
///
/// Returns a `Result<Option<char>>` where:
///
/// - `Ok(Some(char))` means we found a `char`
/// - `Ok(None)` means we hit EOF
/// - `Err(err)` means we encountered some IO error
#[derive(Debug)]
pub struct CharReader<T: BufRead> {
    reader: T,
    buffer: VecDeque<char>,
    read_eof: bool,
}

impl<T: BufRead + 'static> ParserSource for CharReader<T> {}

impl<T: BufRead> Reader for CharReader<T> {
    type Item = char;
    type Err = QError;

    fn read(self) -> (Self, Result<char, QError>) {
        let Self {
            mut reader,
            mut buffer,
            mut read_eof,
        } = self;
        if buffer.is_empty() {
            if read_eof {
                (
                    // TODO throw IO error EOF here?
                    Self {
                        reader,
                        buffer,
                        read_eof,
                    },
                    Err(QError::not_found_err()),
                )
            } else {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            for c in line.chars() {
                                buffer.push_back(c);
                            }
                            let ch = buffer.pop_front().unwrap();
                            (
                                Self {
                                    reader,
                                    buffer,
                                    read_eof,
                                },
                                Ok(ch),
                            )
                        } else {
                            read_eof = true;
                            (
                                Self {
                                    reader,
                                    buffer,
                                    read_eof,
                                },
                                Err(QError::not_found_err()),
                            )
                        }
                    }
                    Err(err) => (
                        Self {
                            reader,
                            buffer,
                            read_eof,
                        },
                        Err(err.into()),
                    ),
                }
            }
        } else {
            let ch = buffer.pop_front().unwrap();
            (
                Self {
                    reader,
                    buffer,
                    read_eof,
                },
                Ok(ch),
            )
        }
    }

    fn undo_item(self, ch: char) -> Self {
        let Self {
            reader,
            mut buffer,
            read_eof,
        } = self;
        buffer.push_front(ch);
        Self {
            reader,
            buffer,
            read_eof,
        }
    }
}

impl<T: BufRead> CharReader<T> {
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            buffer: VecDeque::new(),
            read_eof: false,
        }
    }
}

//
// Naming conventions
//
// read_any  : reads the next item. Err::NotFound is okay.
// read_some : reads the next item. Err::NotFound is mapped to the given error.

//
// Parser combinators
//

pub fn read_any_whitespace<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + 'static,
{
    super::pc::str::take_one_or_more(is_whitespace)
}

pub fn skipping_whitespace_around<P, T, S>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + Undo<String> + Undo<T> + 'static,
    T: 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
{
    map(
        and(skip_whitespace(), and(source, skip_whitespace())),
        |(_, (l, _))| l,
    )
}

pub fn skip_whitespace<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + 'static,
{
    super::pc::str::take_zero_or_more(is_whitespace)
}

pub fn skip_whitespace_eol<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + 'static,
{
    super::pc::str::take_zero_or_more(is_eol_or_whitespace)
}

pub fn skipping_whitespace<P, S, T>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
    T: 'static,
{
    skip_first(skip_whitespace(), source)
}

pub fn skipping_whitespace_lazy<P, S, T>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + Undo<String> + 'static,
    S: Fn() -> Box<dyn Fn(P) -> (P, Result<T, QError>)> + 'static,
    T: 'static,
{
    skip_first_lazy(skip_whitespace(), source)
}

pub fn skipping_whitespace_eol<P, S, T>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
    T: 'static,
{
    skip_first(skip_whitespace_eol(), source)
}

pub fn read_any_symbol<P>() -> Box<dyn Fn(P) -> (P, Result<char, QError>)>
where
    P: ParserSource + 'static,
{
    read_any_if(is_symbol)
}

pub fn read_any_letter<P>() -> Box<dyn Fn(P) -> (P, Result<char, QError>)>
where
    P: ParserSource + 'static,
{
    read_any_if(is_letter)
}
/// Reads any identifier. Note that the result might be a keyword.
/// An identifier must start with a letter and consists of letters, numbers and the dot.
pub fn read_any_identifier<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + 'static,
{
    map(
        if_first_maybe_second(
            read_any_letter(),
            super::pc::str::take_zero_or_more(|ch| {
                (ch >= 'a' && ch <= 'z')
                    || (ch >= 'A' && ch <= 'Z')
                    || (ch >= '0' && ch <= '9')
                    || (ch == '.')
            }),
        ),
        |(l, opt_r)| {
            let mut result: String = String::new();
            result.push(l);
            result.push_str(opt_r.unwrap_or_default().as_ref());
            result
        },
    )
}

/// Reads any keyword.
pub fn read_any_keyword<P>() -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QError>)>
where
    P: ParserSource + Undo<String> + 'static,
{
    switch_from_str(read_any_identifier())
}

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn read_any_word<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + Undo<String> + 'static,
{
    map_or_undo(read_any_identifier(), |s| match Keyword::from_str(&s) {
        Ok(_) => MapOrUndo::Undo(s),
        Err(_) => MapOrUndo::Ok(s),
    })
}

pub fn read_keyword_if<P, F>(
    predicate: F,
) -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QError>)>
where
    P: ParserSource + Undo<String> + Undo<(Keyword, String)> + 'static,
    F: Fn(Keyword) -> bool + 'static,
{
    super::pc::common::filter_any(read_any_keyword(), move |(k, _)| predicate(*k))
}

pub fn try_read_keyword<P>(
    needle: Keyword,
) -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QError>)>
where
    P: ParserSource + Undo<String> + Undo<(Keyword, String)> + 'static,
{
    read_keyword_if(move |k| k == needle)
}

pub fn demand_keyword<P>(
    needle: Keyword,
) -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QError>)>
where
    P: ParserSource + Undo<String> + Undo<(Keyword, String)> + HasLocation + 'static,
{
    demand(
        super::pc::common::filter_any(read_any_keyword(), move |(k, _)| *k == needle),
        move || QError::SyntaxError(format!("Expected keyword {}", needle)),
    )
}

pub fn read_any_eol<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + 'static,
{
    super::pc::str::take_one_or_more(is_eol)
}

pub fn read_any_eol_whitespace<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + 'static,
{
    super::pc::str::take_one_or_more(is_eol_or_whitespace)
}

pub fn read_any_digits<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + 'static,
{
    super::pc::str::take_one_or_more(is_digit)
}

pub fn default_if_predicate<P, T, F>(predicate: F) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + HasLocation + 'static,
    T: Default + 'static,
    F: Fn(char) -> bool + 'static,
{
    Box::new(move |reader| {
        let (reader, next) = reader.read();
        match next {
            Ok(ch) => {
                if predicate(ch) {
                    (reader.undo_item(ch), Ok(T::default()))
                } else {
                    reader.undo_and_err_not_found(ch)
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    // EOF is ok
                    (reader, Ok(T::default()))
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

//
// Combine two or more parsers
//

pub fn and<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn(P) -> (P, Result<T2, E>) + 'static,
    P: ParserSource + Undo<T1> + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(r1) => {
                let (char_reader, res2) = second(char_reader);
                match res2 {
                    Ok(r2) => (char_reader, Ok((r1, r2))),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (char_reader.undo(r1), Err(err))
                        } else {
                            (char_reader, Err(err))
                        }
                    }
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

// internal use only
fn and_no_undo<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn(P) -> (P, Result<T2, E>) + 'static,
    P: ParserSource + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(r1) => {
                let (char_reader, res2) = second(char_reader);
                match res2 {
                    Ok(r2) => (char_reader, Ok((r1, r2))),
                    Err(err) => (char_reader, Err(err)),
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

// internal use only
fn and_no_undo_lazy<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn() -> Box<dyn Fn(P) -> (P, Result<T2, E>)> + 'static,
    P: ParserSource + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(r1) => {
                let (char_reader, res2) = second()(char_reader);
                match res2 {
                    Ok(r2) => (char_reader, Ok((r1, r2))),
                    Err(err) => (char_reader, Err(err)),
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn maybe_first_and_second_no_undo<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(Option<T1>, T2), E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn(P) -> (P, Result<T2, E>) + 'static,
    P: ParserSource + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |reader| {
        let (reader, res1) = first(reader);
        match res1 {
            Ok(r1) => {
                let (reader, res2) = second(reader);
                match res2 {
                    Ok(r2) => (reader, Ok((Some(r1), r2))),
                    Err(err) => (reader, Err(err)),
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    let (reader, res2) = second(reader);
                    match res2 {
                        Ok(r2) => (reader, Ok((None, r2))),
                        Err(err) => (reader, Err(err)),
                    }
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

pub fn maybe_first_lazy_and_second_no_undo<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(Option<T1>, T2), E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn() -> Box<dyn Fn(P) -> (P, Result<T1, E>)> + 'static,
    F2: Fn(P) -> (P, Result<T2, E>) + 'static,
    P: ParserSource + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |reader| {
        let (reader, res1) = first()(reader);
        match res1 {
            Ok(r1) => {
                let (reader, res2) = second(reader);
                match res2 {
                    Ok(r2) => (reader, Ok((Some(r1), r2))),
                    Err(err) => (reader, Err(err)),
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    let (reader, res2) = second(reader);
                    match res2 {
                        Ok(r2) => (reader, Ok((None, r2))),
                        Err(err) => (reader, Err(err)),
                    }
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

pub fn if_first_demand_second<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QError>)>
where
    P: ParserSource + HasLocation + 'static,
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, QError>) + 'static,
    F2: Fn(P) -> (P, Result<T2, QError>) + 'static,
    FE: Fn() -> QError + 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(r1) => {
                let (char_reader, res2) = second(char_reader);
                match res2 {
                    Ok(r2) => (char_reader, Ok((r1, r2))),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (char_reader, Err(err_fn()))
                        } else {
                            (char_reader, Err(err))
                        }
                    }
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn if_first_demand_second_lazy<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QError>)>
where
    P: ParserSource + HasLocation + 'static,
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, QError>) + 'static,
    F2: Fn() -> Box<dyn Fn(P) -> (P, Result<T2, QError>)> + 'static,
    FE: Fn() -> QError + 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(r1) => {
                let (char_reader, res2) = second()(char_reader);
                match res2 {
                    Ok(r2) => (char_reader, Ok((r1, r2))),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (char_reader, Err(err_fn()))
                        } else {
                            (char_reader, Err(err))
                        }
                    }
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn if_first_maybe_second<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(T1, Option<T2>), E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn(P) -> (P, Result<T2, E>) + 'static,
    E: IsNotFoundErr,
    P: ParserSource + 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(r1) => {
                let (char_reader, res2) = second(char_reader);
                match res2 {
                    Ok(r2) => (char_reader, Ok((r1, Some(r2)))),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (char_reader, Ok((r1, None)))
                        } else {
                            (char_reader, Err(err))
                        }
                    }
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn if_first_maybe_second_peeking_first<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(T1, Option<T2>), E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn(P, &T1) -> (P, Result<T2, E>) + 'static,
    E: IsNotFoundErr,
    P: ParserSource + 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(r1) => {
                let (char_reader, res2) = second(char_reader, &r1);
                match res2 {
                    Ok(r2) => (char_reader, Ok((r1, Some(r2)))),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (char_reader, Ok((r1, None)))
                        } else {
                            (char_reader, Err(err))
                        }
                    }
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn or<P, F1, F2, T, E>(first: F1, second: F2) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
where
    F1: Fn(P) -> (P, Result<T, E>) + 'static,
    F2: Fn(P) -> (P, Result<T, E>) + 'static,
    E: IsNotFoundErr,
    P: ParserSource + 'static,
{
    Box::new(move |reader| {
        let (reader, res1) = first(reader);
        match res1 {
            Ok(ch) => (reader, Ok(ch)),
            Err(err) => {
                if err.is_not_found_err() {
                    second(reader)
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

pub fn or_vec<P, T, E, F>(mut sources: Vec<F>) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
where
    P: ParserSource + 'static,
    T: 'static,
    E: IsNotFoundErr + 'static,
    F: Fn(P) -> (P, Result<T, E>) + 'static,
{
    if sources.len() > 2 {
        let first = sources.remove(0);
        or(first, or_vec(sources))
    } else if sources.len() == 2 {
        let second = sources.pop().unwrap();
        let first = sources.pop().unwrap();
        or(first, second)
    } else {
        panic!("or_vec must have at least two functions to choose from");
    }
}

/// Skips the result of the first parser and returns the result of the second one.
/// The only case where the result of the first parser is returned is if
/// it returns an unrecoverable error.
pub fn skip_first<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<T2, E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn(P) -> (P, Result<T2, E>) + 'static,
    P: ParserSource + Undo<T1> + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |reader| {
        let (reader, first_result) = first(reader);
        match first_result {
            Ok(first_ok) => {
                let (reader, second_result) = second(reader);
                match second_result {
                    Ok(ch) => (reader, Ok(ch)),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (reader.undo(first_ok), Err(err))
                        } else {
                            (reader, Err(err))
                        }
                    }
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    second(reader)
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

pub fn skip_first_lazy<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<T2, E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn() -> Box<dyn Fn(P) -> (P, Result<T2, E>)> + 'static,
    P: ParserSource + Undo<T1> + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |reader| {
        let (reader, first_result) = first(reader);
        match first_result {
            Ok(first_ok) => {
                let (reader, second_result) = second()(reader);
                match second_result {
                    Ok(ch) => (reader, Ok(ch)),
                    Err(err) => {
                        if err.is_not_found_err() {
                            (reader.undo(first_ok), Err(err))
                        } else {
                            (reader, Err(err))
                        }
                    }
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    second()(reader)
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

pub fn abort_if<P, T1, T2, E, F1, F2>(
    predicate_source: F1,
    source: F2,
) -> Box<dyn Fn(P) -> (P, Result<T2, E>)>
where
    P: ParserSource + Undo<()> + Undo<T1> + 'static,
    T1: 'static,
    T2: 'static,
    E: NotFoundErr + 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn(P) -> (P, Result<T2, E>) + 'static,
{
    map(and(negate(predicate_source), source), |(_, r)| r)
}

pub fn negate<P, T, E, S>(source: S) -> Box<dyn Fn(P) -> (P, Result<(), E>)>
where
    P: ParserSource + Undo<T> + 'static,
    T: 'static,
    E: NotFoundErr + 'static,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
{
    Box::new(move |reader| {
        let (reader, res) = source(reader);
        match res {
            Ok(x) => (reader.undo(x), Err(E::not_found_err())),
            Err(err) => {
                if err.is_not_found_err() {
                    (reader, Ok(()))
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

/// Combines the two given parsers, demanding that there is some whitespace between
/// their results.
/// If the first parser succeeds, there must be a whitespace after it and the
/// second parser must also succeed.
pub fn with_some_whitespace_between<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QError>)>
where
    P: ParserSource + HasLocation + 'static,
    F1: Fn(P) -> (P, Result<T1, QError>) + 'static,
    F2: Fn(P) -> (P, Result<T2, QError>) + 'static,
    T1: 'static,
    T2: 'static,
    FE: Fn() -> QError + 'static,
{
    map(
        if_first_demand_second(first, and_no_undo(read_any_whitespace(), second), err_fn),
        |(l, (_, r))| (l, r),
    )
}

pub fn with_some_whitespace_between_lazy<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QError>)>
where
    P: ParserSource + HasLocation + 'static,
    F1: Fn(P) -> (P, Result<T1, QError>) + 'static,
    F2: Fn() -> Box<dyn Fn(P) -> (P, Result<T2, QError>)> + 'static,
    T1: 'static,
    T2: 'static,
    FE: Fn() -> QError + 'static,
{
    map(
        if_first_demand_second(
            first,
            and_no_undo_lazy(read_any_whitespace(), second),
            err_fn,
        ),
        |(l, (_, r))| (l, r),
    )
}

/// Combines the two given parsers, demanding that there is some whitespace before
/// the first result as well as between the two parsed results.
/// If the first parser succeeds, the second parser must also succeed.
///
/// Returns not found if there is no leading whitespace or if the first parser fails.
pub fn with_some_whitespace_before_and_between<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    F1: Fn(P) -> (P, Result<T1, QError>) + 'static,
    F2: Fn(P) -> (P, Result<T2, QError>) + 'static,
    T1: 'static,
    T2: 'static,
    FE: Fn() -> QError + 'static,
{
    map(
        and(
            read_any_whitespace(),
            with_some_whitespace_between(first, second, err_fn),
        ),
        |(_, r)| r,
    )
}

/// Combines the two given parsers, allowing some optional whitespace between their results.
/// If the first parser succeeds, the second must also succeed.
pub fn with_any_whitespace_between<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    F1: Fn(P) -> (P, Result<T1, QError>) + 'static,
    F2: Fn(P) -> (P, Result<T2, QError>) + 'static,
    T1: 'static,
    T2: 'static,
    FE: Fn() -> QError + 'static,
{
    if_first_demand_second(first, skipping_whitespace(second), err_fn)
}

//
// Modify the result of a parser
//

fn map_all<P, S, M, T, U, E>(source: S, mapper: M) -> Box<dyn Fn(P) -> (P, Result<U, E>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
    M: Fn(P, Result<T, E>) -> (P, Result<U, E>) + 'static,
    T: 'static,
    U: 'static,
    E: 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, res) = source(char_reader);
        mapper(char_reader, res)
    })
}

/// Maps the ok output of the `source` with the given mapper function.
/// The mapper function has total control over the result, as it receives both
/// the ok output of the source and the reader. This is the most flexible mapper
/// function.
pub fn map_to_reader<P, S, M, T, U, E>(source: S, mapper: M) -> Box<dyn Fn(P) -> (P, Result<U, E>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
    M: Fn(P, T) -> (P, Result<U, E>) + 'static,
    T: 'static,
    U: 'static,
    E: 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => mapper(char_reader, ch),
            Err(err) => (char_reader, Err(err)),
        }
    })
}

/// Map the result of the source using the given mapper function.
/// Be careful as it will not undo if the mapper function returns a Not Found result.
pub fn map_to_result_no_undo<P, S, M, T, U, E>(
    source: S,
    mapper: M,
) -> Box<dyn Fn(P) -> (P, Result<U, E>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
    M: Fn(T) -> Result<U, E> + 'static,
    T: 'static,
    U: 'static,
    E: 'static,
{
    map_to_reader(source, move |reader, ok| (reader, mapper(ok)))
}

pub enum MapOrUndo<T, U> {
    Ok(T),
    Undo(U),
}

/// Maps the ok output of the `source` with the given mapper function.
/// The function can convert an ok result into a not found result.
pub fn map_or_undo<P, S, M, T, U, E>(source: S, mapper: M) -> Box<dyn Fn(P) -> (P, Result<U, E>)>
where
    P: ParserSource + 'static + Undo<T>,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
    M: Fn(T) -> MapOrUndo<U, T> + 'static,
    T: 'static,
    U: 'static,
    E: NotFoundErr + 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => {
                // switch it
                match mapper(ch) {
                    MapOrUndo::Ok(x) => (char_reader, Ok(x)),
                    MapOrUndo::Undo(x) => (char_reader.undo(x), Err(E::not_found_err())),
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn map<P, S, M, R, U, E>(source: S, mapper: M) -> Box<dyn Fn(P) -> (P, Result<U, E>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<R, E>) + 'static,
    M: Fn(R) -> U + 'static,
    R: 'static,
    U: 'static,
    E: 'static,
{
    map_to_result_no_undo(source, move |x| Ok(mapper(x)))
}

pub fn switch_from_str<P, S, T, E>(source: S) -> Box<dyn Fn(P) -> (P, Result<(T, String), E>)>
where
    P: ParserSource + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<String, E>) + 'static,
    T: FromStr + 'static,
    E: NotFoundErr + 'static,
{
    Box::new(move |reader| {
        let (reader, next) = source(reader);
        match next {
            Ok(s) => match T::from_str(&s) {
                Ok(u) => (reader, Ok((u, s))),
                Err(_) => (reader.undo(s), Err(E::not_found_err())),
            },
            Err(err) => (reader, Err(err)),
        }
    })
}

//
// Take multiple items
//

pub fn take_zero_or_more_to_default<P, S, T, F>(
    source: S,
    is_terminal: F,
) -> Box<dyn Fn(P) -> (P, Result<Vec<T>, QError>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
    T: 'static,
    F: Fn(&T) -> bool + 'static,
{
    map_all(
        take_one_or_more(source, is_terminal, QError::not_found_err),
        |reader, res| match res {
            Ok(v) => (reader, Ok(v)),
            Err(err) => {
                if err.is_not_found_err() {
                    (reader, Ok(vec![]))
                } else {
                    (reader, Err(err))
                }
            }
        },
    )
}

pub fn take_zero_or_more<P, S, T, F>(
    source: S,
    is_terminal: F,
) -> Box<dyn Fn(P) -> (P, Result<Vec<T>, QError>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
    T: 'static,
    F: Fn(&T) -> bool + 'static,
{
    take_one_or_more(source, is_terminal, QError::not_found_err)
}

pub fn take_one_or_more<P, S, T, F, FE>(
    source: S,
    is_terminal: F,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<Vec<T>, QError>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
    F: Fn(&T) -> bool + 'static,
    FE: Fn() -> QError + 'static,
{
    Box::new(move |char_reader| {
        let mut result: Vec<T> = vec![];
        let mut cr: P = char_reader;
        loop {
            let (x, next) = source(cr);
            cr = x;
            match next {
                Err(err) => {
                    if err.is_not_found_err() {
                        break;
                    } else {
                        return (cr, Err(err));
                    }
                }
                Ok(ch) => {
                    let last = is_terminal(&ch);
                    result.push(ch);
                    if last {
                        break;
                    }
                }
            }
        }
        if result.is_empty() {
            (cr, Err(err_fn()))
        } else {
            (cr, Ok(result))
        }
    })
}

pub fn csv_one_or_more<P, S, R, FE>(
    source: S,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<R, QError>) + 'static,
    R: 'static,
    FE: Fn() -> QError + 'static,
{
    map(
        take_one_or_more(
            if_first_maybe_second(
                skipping_whitespace(source),
                skipping_whitespace(with_pos(try_read(','))),
            ),
            |x| x.1.is_none(),
            err_fn,
        ),
        |x| x.into_iter().map(|x| x.0).collect(),
    )
}

pub fn csv_one_or_more_lazy<P, S, R, FE>(
    source: S,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    S: Fn() -> Box<dyn Fn(P) -> (P, Result<R, QError>)> + 'static,
    R: 'static,
    FE: Fn() -> QError + 'static,
{
    map(
        take_one_or_more(
            if_first_maybe_second(
                skipping_whitespace_lazy(source),
                skipping_whitespace(with_pos(try_read(','))),
            ),
            |x| x.1.is_none(),
            err_fn,
        ),
        |x| x.into_iter().map(|x| x.0).collect(),
    )
}

pub fn csv_zero_or_more<P, S, R>(source: S) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<R, QError>) + 'static,
    R: 'static,
{
    map(
        take_zero_or_more(
            if_first_maybe_second(
                skipping_whitespace(source),
                skipping_whitespace(with_pos(try_read(','))),
            ),
            |x| x.1.is_none(),
        ),
        |x| x.into_iter().map(|x| x.0).collect(),
    )
}

pub fn csv_zero_or_more_lazy<P, S, R>(source: S) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    S: Fn() -> Box<dyn Fn(P) -> (P, Result<R, QError>)> + 'static,
    R: 'static,
{
    map(
        take_zero_or_more(
            if_first_maybe_second(
                skipping_whitespace_lazy(source),
                skipping_whitespace(with_pos(try_read(','))),
            ),
            |x| x.1.is_none(),
        ),
        |x| x.into_iter().map(|x| x.0).collect(),
    )
}

pub fn in_parenthesis<P, T, S>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
    T: 'static,
{
    map_to_result_no_undo(
        and(
            try_read('('),
            maybe_first_and_second_no_undo(
                source,
                demand(try_read(')'), || {
                    QError::SyntaxError("Expected closing parenthesis".to_string())
                }),
            ),
        ),
        |(_, (r, _))| match r {
            Some(x) => Ok(x),
            None => Err(QError::not_found_err()),
        },
    )
}

pub fn in_parenthesis_lazy<P, T, S>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn() -> Box<dyn Fn(P) -> (P, Result<T, QError>)> + 'static,
    T: 'static,
{
    map_to_result_no_undo(
        and(
            try_read('('),
            maybe_first_lazy_and_second_no_undo(
                source,
                demand(try_read(')'), || {
                    QError::SyntaxError("Expected closing parenthesis".to_string())
                }),
            ),
        ),
        |(_, (r, _))| match r {
            Some(x) => Ok(x),
            None => Err(QError::not_found_err()),
        },
    )
}

//
// Keyword guards
//

pub fn with_keyword_before<P, T, S>(
    needle: Keyword,
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + Undo<(Keyword, String)> + 'static,
    T: 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
{
    map(
        with_some_whitespace_between(try_read_keyword(needle), source, move || {
            QError::SyntaxError(format!("Cannot parse after {}", needle))
        }),
        |(_, r)| r,
    )
}

pub fn with_keyword_after<P, T, S, FE>(
    source: S,
    needle: Keyword,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + Undo<(Keyword, String)> + 'static,
    T: 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
    FE: Fn() -> QError + 'static,
{
    // TODO remove the skipping_whitespace , remove with_keyword_after altogether
    map(
        if_first_demand_second(
            source,
            skipping_whitespace(try_read_keyword(needle)),
            err_fn,
        ),
        |(l, _)| l,
    )
}

pub fn with_two_keywords<P, T, S>(
    first: Keyword,
    second: Keyword,
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<T, QError>)>
where
    P: ParserSource + HasLocation + Undo<String> + Undo<(Keyword, String)> + 'static,
    T: 'static,
    S: Fn(P) -> (P, Result<T, QError>) + 'static,
{
    with_keyword_before(first, with_keyword_before(second, source))
}

//
// EolReader
//

pub struct EolReader<T: BufRead> {
    char_reader: CharReader<T>,
    pos: Location,
    line_lengths: Vec<u32>,
}

// Location tracking + treating CRLF as one char
impl<T: BufRead> EolReader<T> {
    pub fn new(char_reader: CharReader<T>) -> Self {
        Self {
            char_reader,
            pos: Location::start(),
            line_lengths: vec![],
        }
    }
}

impl<T: BufRead + 'static> ParserSource for EolReader<T> {}

impl<T: BufRead + 'static> Reader for EolReader<T> {
    type Item = char;
    type Err = QError;

    fn read(self) -> (Self, Result<char, QError>) {
        let Self {
            char_reader,
            mut pos,
            mut line_lengths,
        } = self;
        let (char_reader, next) = or(
            or(
                try_read('\n'),
                map(
                    // Tradeoff: CRLF becomes just CR
                    // Alternatives:
                    // - Return a String instead of a char
                    // - Return a new enum type instead of a char
                    // - Encode CRLF as a special char e.g. CR = 13 + LF = 10 -> CRLF = 23
                    if_first_maybe_second(try_read('\r'), try_read('\n')),
                    |(cr, _)| cr,
                ),
            ),
            read_any(),
        )(char_reader);
        match next {
            Ok('\r') | Ok('\n') => {
                if line_lengths.len() + 1 == (pos.row() as usize) {
                    line_lengths.push(pos.col());
                }
                pos.inc_row();
            }
            Ok(_) => {
                pos.inc_col();
            }
            _ => {}
        }
        (
            Self {
                char_reader,
                pos,
                line_lengths,
            },
            next,
        )
    }

    fn undo_item(self, x: char) -> Self {
        let Self {
            mut char_reader,
            mut pos,
            line_lengths,
        } = self;
        match x {
            '\r' | '\n' => {
                pos = Location::new(pos.row() - 1, line_lengths[(pos.row() - 2) as usize]);
                char_reader = char_reader.undo_item(x);
            }
            _ => {
                pos = Location::new(pos.row(), pos.col() - 1);
                char_reader = char_reader.undo_item(x);
            }
        }
        Self {
            char_reader,
            pos,
            line_lengths,
        }
    }
}

impl<T: BufRead + 'static> Undo<String> for EolReader<T> {
    fn undo(self, s: String) -> Self {
        let mut result = self;
        for ch in s.chars().rev() {
            result = result.undo(ch);
        }
        result
    }
}

impl<T: BufRead + 'static> Undo<CaseInsensitiveString> for EolReader<T> {
    fn undo(self, s: CaseInsensitiveString) -> Self {
        let inner: String = s.into();
        self.undo(inner)
    }
}

impl<T: BufRead + 'static> Undo<(Keyword, String)> for EolReader<T> {
    fn undo(self, s: (Keyword, String)) -> Self {
        self.undo(s.1)
    }
}

impl<T: BufRead> HasLocation for EolReader<T> {
    fn pos(&self) -> Location {
        self.pos
    }
}

//
// Converters from str and File
//

// bytes || &str -> CharReader
impl<T> From<T> for CharReader<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        CharReader::new(BufReader::new(Cursor::new(input)))
    }
}

// File -> CharReader
impl From<File> for CharReader<BufReader<File>> {
    fn from(input: File) -> Self {
        CharReader::new(BufReader::new(input))
    }
}

// bytes || &str -> EolReader
impl<T> From<T> for EolReader<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        EolReader::new(CharReader::new(BufReader::new(Cursor::new(input))))
    }
}

// File -> EolReader
impl From<File> for EolReader<BufReader<File>> {
    fn from(input: File) -> Self {
        EolReader::new(CharReader::new(BufReader::new(input)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eof_is_twice() {
        let reader: CharReader<BufReader<Cursor<&str>>> = "123".into();
        let (reader, next) = reader.read();
        assert_eq!(next.unwrap(), '1');
        let (reader, next) = reader.read();
        assert_eq!(next.unwrap(), '2');
        let (reader, next) = reader.read();
        assert_eq!(next.unwrap(), '3');
        let (reader, next) = reader.read();
        assert_eq!(next.is_err(), true);
        let (_, next) = reader.read();
        assert_eq!(next.is_err(), true);
    }
}
