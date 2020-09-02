use crate::common::{CaseInsensitiveString, HasLocation, Location, QError};
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::pc::map::{map, source_and_then_some};
use crate::parser::pc::*;
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

impl<T: BufRead> Reader for CharReader<T> {
    type Item = char;
    type Err = QError;

    fn read(self) -> ReaderResult<Self, char, QError> {
        let Self {
            mut reader,
            mut buffer,
            mut read_eof,
        } = self;
        if buffer.is_empty() {
            if read_eof {
                Ok((
                    // TODO throw IO error EOF here?
                    Self {
                        reader,
                        buffer,
                        read_eof,
                    },
                    None,
                ))
            } else {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            for c in line.chars() {
                                buffer.push_back(c);
                            }
                            let ch = buffer.pop_front().unwrap();
                            Ok((
                                Self {
                                    reader,
                                    buffer,
                                    read_eof,
                                },
                                Some(ch),
                            ))
                        } else {
                            read_eof = true;
                            Ok((
                                Self {
                                    reader,
                                    buffer,
                                    read_eof,
                                },
                                None,
                            ))
                        }
                    }
                    Err(err) => Err((
                        Self {
                            reader,
                            buffer,
                            read_eof,
                        },
                        err.into(),
                    )),
                }
            }
        } else {
            let ch = buffer.pop_front().unwrap();
            Ok((
                Self {
                    reader,
                    buffer,
                    read_eof,
                },
                Some(ch),
            ))
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
// Parser combinators
//

pub fn read_any_symbol<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, char, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
{
    read_if(is_symbol)
}

pub fn read_any_letter<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, char, E>>
where
R: Reader<Item = char, Err = E> + 'static,
{
    read_if(is_letter)
}

/// Reads any identifier. Note that the result might be a keyword.
/// An identifier must start with a letter and consists of letters, numbers and the dot.
pub fn read_any_identifier<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
where
R: Reader<Item = char, Err = E> + 'static,
E: 'static,
{
    map_default_to_not_found(super::pc::str::zero_or_more_if_leading_remaining(
        is_letter,
        is_non_leading_identifier,
    ))
}

/// Reads any keyword.
pub fn read_any_keyword<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
where
R: Reader<Item = char, Err = E> + 'static,
E: 'static,
{
    crate::parser::pc::str::switch_from_str(read_any_identifier())
}

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn read_any_word<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R,  String, E>>
where
R: Reader<Item = char, Err = E> + 'static,
E: 'static
{
    source_and_then_some(
        read_any_identifier(),
        |reader: R, s| match Keyword::from_str(&s) {
            Ok(_) => Ok((reader.undo(s), None)),
            Err(_) => Ok((reader, Some(s))),
        },
    )
}

pub fn read_keyword_if<R, E, F>(
    predicate: F,
) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    F: Fn(Keyword) -> bool + 'static,
E: 'static
{
    super::pc::common::filter(read_any_keyword(), move |(k, _)| predicate(*k))
}

// TODO optimize
pub fn try_read_keyword<R, E>(
    needle: Keyword,
) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
where
R: Reader<Item = char, Err = E> + 'static,
E: 'static
{
    read_keyword_if(move |k| k == needle)
}

pub fn demand_keyword<T: BufRead + 'static>(
    needle: Keyword,
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (Keyword, String), QError>> {
    demand(
        try_read_keyword(needle),
        QError::syntax_error_fn(format!("Expected: {}", needle)),
    )
}

pub fn demand_guarded_keyword<T: BufRead + 'static>(
    needle: Keyword,
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (Keyword, String), QError>> {
    drop_left(and(
        demand(
            crate::parser::pc::ws::one_or_more(),
            QError::syntax_error_fn(format!("Expected: whitespace before {}", needle)),
        ),
        demand_keyword(needle),
    ))
}

pub fn read_any_digits<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R,  String, E>>
where
R: Reader<Item = char, Err = E> + 'static,
E: 'static
{
    super::pc::str::one_or_more_if(is_digit)
}

//
// Modify the result of a parser
//

//
// Take multiple items
//

pub fn csv_zero_or_more<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, Vec<T>, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    S: Fn(R) -> ReaderResult<R, T, E> + 'static,
    T: 'static,
    E: 'static
{
    zero_or_more(opt_seq2(
        source,
        crate::parser::pc::ws::zero_or_more_around(try_read(',')),
    ))
}

pub fn in_parenthesis<R, S, T>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T, QError>>
where
    R: Reader<Item = char, Err = QError> + 'static,
    S: Fn(R) -> ReaderResult<R, T, QError> + 'static,
    T: 'static,
{
    map(
        seq3(
            try_read('('),
            source,
            demand(
                try_read(')'),
                QError::syntax_error_fn("Expected: closing parenthesis"),
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

impl<T: BufRead + 'static> Reader for EolReader<T> {
    type Item = char;
    type Err = QError;

    fn read(self) -> ReaderResult<Self, char, QError> {
        let Self {
            char_reader,
            mut pos,
            mut line_lengths,
        } = self;
        let res = or(
            or(
                try_read('\n'),
                map(
                    // Tradeoff: CRLF becomes just CR
                    // Alternatives:
                    // - Return a String instead of a char
                    // - Return a new enum type instead of a char
                    // - Encode CRLF as a special char e.g. CR = 13 + LF = 10 -> CRLF = 23
                    opt_seq2(try_read('\r'), try_read('\n')),
                    |(cr, _)| cr,
                ),
            ),
            read(),
        )(char_reader);
        match res {
            Ok((char_reader, opt_res)) => {
                match &opt_res {
                    Some('\r') | Some('\n') => {
                        if line_lengths.len() + 1 == (pos.row() as usize) {
                            line_lengths.push(pos.col());
                        }
                        pos.inc_row();
                    }
                    Some(_) => {
                        pos.inc_col();
                    }
                    _ => {}
                }

                Ok((
                    Self {
                        char_reader,
                        pos,
                        line_lengths,
                    },
                    opt_res,
                ))
            }
            Err((char_reader, err)) => Err((
                Self {
                    char_reader,
                    pos,
                    line_lengths,
                },
                err,
            )),
        }
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

impl<T: BufRead> HasLocation for EolReader<T> {
    fn pos(&self) -> Location {
        self.pos
    }
}

// ========================================================
// Undo support
// ========================================================

impl<R: Reader<Item = char>> Undo<char> for R {
    fn undo(self, item: char) -> Self {
        self.undo_item(item)
    }
}

impl<R: Reader<Item = char>> Undo<TypeQualifier> for R {
    fn undo(self, s: TypeQualifier) -> Self {
        let ch: char = s.try_into().unwrap();
        self.undo_item(ch)
    }
}

impl<R: Reader<Item = char>> Undo<String> for R {
    fn undo(self, s: String) -> Self {
        let mut result = self;
        for ch in s.chars().rev() {
            result = result.undo_item(ch);
        }
        result
    }
}

impl<R: Reader<Item = char>> Undo<CaseInsensitiveString> for R {
    fn undo(self, s: CaseInsensitiveString) -> Self {
        let inner: String = s.into();
        self.undo(inner)
    }
}

impl<R: Reader<Item = char>> Undo<Name> for R {
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

impl<R: Reader<Item = char>> Undo<(Keyword, String)> for R {
    fn undo(self, s: (Keyword, String)) -> Self {
        self.undo(s.1)
    }
}

// ========================================================
// Converters from str and File
// ========================================================

// bytes || &str -> CharReader
impl<T> From<T> for CharReader<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        CharReader::new(BufReader::new(Cursor::new(input)))
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
        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '1');
        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '2');
        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.unwrap(), '3');
        let (reader, next) = reader.read().unwrap();
        assert_eq!(next.is_some(), false);
        let (_, next) = reader.read().unwrap();
        assert_eq!(next.is_some(), false);
    }
}
