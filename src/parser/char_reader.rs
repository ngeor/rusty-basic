use crate::common::{CaseInsensitiveString, HasLocation, Location, QError};
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
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

fn is_non_leading_identifier(ch: char) -> bool {
    (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || (ch >= '0' && ch <= '9') || (ch == '.')
}

fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
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
    map_default_to_not_found(super::pc::str::zero_or_more_if_leading_remaining(
        is_letter,
        is_non_leading_identifier,
    ))
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

// TODO optimize
pub fn try_read_keyword<P>(
    needle: Keyword,
) -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QError>)>
where
    P: ParserSource + Undo<String> + Undo<(Keyword, String)> + 'static,
{
    read_keyword_if(move |k| k == needle)
}

pub fn read_any_digits<P>() -> Box<dyn Fn(P) -> (P, Result<String, QError>)>
where
    P: ParserSource + 'static,
{
    super::pc::str::one_or_more_if(is_digit)
}

//
// Combine two or more parsers
//

#[deprecated]
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

#[deprecated]
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

//
// Modify the result of a parser
//

pub enum MapOrUndo<T, U> {
    Ok(T),
    Undo(U),
}

/// Maps the ok output of the `source` with the given mapper function.
/// The function can convert an ok result into a not found result.
#[deprecated]
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
        (cr, Ok(result))
    })
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
                source,
                crate::parser::pc::ws::zero_or_more_around(try_read(',')),
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
    map(
        seq3(
            try_read('('),
            source,
            demand(
                try_read(')'),
                QError::syntax_error_fn("Expected closing parenthesis"),
            ),
        ),
        |(_, r, _)| r,
    )
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
            read(),
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
