use crate::common::{
    AtLocation, ErrorEnvelope, HasLocation, Locatable, Location, PeekOptCopy, QError, QErrorNode,
    ReadOpt, ToLocatableError,
};
use crate::lexer::{Keyword, Lexeme};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::str::FromStr;

//
// NotFoundErr
//

pub trait NotFoundErr {
    fn is_not_found_err(&self) -> bool;
    fn not_found_err() -> Self;
}

impl NotFoundErr for QError {
    fn is_not_found_err(&self) -> bool {
        *self == QError::CannotParse
    }

    fn not_found_err() -> Self {
        QError::CannotParse
    }
}

//
// ParserSource
//

pub trait ParserSource: Sized {
    type Item;
    type Err;

    fn read(self) -> (Self, Result<Self::Item, Self::Err>);

    fn undo_item(self, item: Self::Item) -> Self;
}

//
// Undo
//

pub trait Undo<T> {
    fn undo(self, item: T) -> Self;
}

impl<P: ParserSource> Undo<P::Item> for P {
    fn undo(self, item: P::Item) -> Self {
        self.undo_item(item)
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

impl<T: BufRead> ParserSource for CharReader<T> {
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

    #[deprecated]
    fn fill_buffer_if_empty(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            self.fill_buffer()
        } else {
            Ok(())
        }
    }

    #[deprecated]
    fn fill_buffer(&mut self) -> std::io::Result<()> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;
        if bytes_read > 0 {
            for c in line.chars() {
                self.buffer.push_back(c);
            }
        }
        Ok(())
    }
}

impl<T: BufRead> ReadOpt for CharReader<T> {
    type Item = char;
    type Err = QErrorNode;

    fn read_ng(&mut self) -> Result<Option<char>, QErrorNode> {
        if self.read_eof {
            Ok(None)
        } else {
            match self.fill_buffer_if_empty() {
                Ok(()) => {
                    if self.buffer.is_empty() {
                        self.read_eof = true;
                        Ok(None)
                    } else {
                        Ok(self.buffer.pop_front())
                    }
                }
                Err(err) => Err(QErrorNode::NoPos(err.into())),
            }
        }
    }
}

impl<T: BufRead> PeekOptCopy for CharReader<T> {
    fn peek_copy_ng(&mut self) -> Result<Option<char>, QErrorNode> {
        if self.read_eof {
            Ok(None)
        } else {
            match self.fill_buffer_if_empty() {
                Ok(_) => {
                    if self.buffer.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(self.buffer[0]))
                    }
                }
                Err(err) => Err(QErrorNode::NoPos(err.into())),
            }
        }
    }
}

//
// Parser combinators
//

pub fn take_any<P>() -> Box<dyn Fn(P) -> (P, Result<P::Item, P::Err>)>
where
    P: ParserSource + 'static,
{
    Box::new(|char_reader| char_reader.read())
}

pub fn take_char<P>(needle: char) -> Box<dyn Fn(P) -> (P, Result<P::Item, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,
    P::Err: NotFoundErr,
{
    take_char_if(move |ch| ch == needle)
}

pub fn filter<P, S, F, T, E>(predicate: F, source: S) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
where
    P: ParserSource + Undo<T> + 'static,
    E: NotFoundErr,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
    F: Fn(&T) -> bool + 'static,
{
    Box::new(move |reader| {
        let (reader, result) = source(reader);
        match result {
            Ok(ch) => {
                if predicate(&ch) {
                    (reader, Ok(ch))
                } else {
                    (reader.undo(ch), Err(E::not_found_err()))
                }
            }
            Err(err) => (reader, Err(err)),
        }
    })
}

pub fn filter_copy<P, S, F, T, E>(predicate: F, source: S) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
where
    P: ParserSource + Undo<T> + 'static,
    E: NotFoundErr,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
    F: Fn(T) -> bool + 'static,
    T: Copy + 'static,
{
    Box::new(move |reader| {
        let (reader, result) = source(reader);
        match result {
            Ok(ch) => {
                if predicate(ch) {
                    (reader, Ok(ch))
                } else {
                    (reader.undo(ch), Err(E::not_found_err()))
                }
            }
            Err(err) => (reader, Err(err)),
        }
    })
}

pub fn take_char_if<P, F>(predicate: F) -> Box<dyn Fn(P) -> (P, Result<P::Item, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,
    F: Fn(char) -> bool + 'static,
    P::Err: NotFoundErr,
{
    filter_copy(predicate, take_any())
}

pub fn and<P, F1, F2, T1, T2>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), P::Err>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, P::Err>) + 'static,
    F2: Fn(P) -> (P, Result<T2, P::Err>) + 'static,
    P: ParserSource + Undo<T1> + 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(r1) => {
                let (char_reader, res2) = second(char_reader);
                match res2 {
                    Ok(r2) => (char_reader, Ok((r1, r2))),
                    Err(err) => (char_reader.undo(r1), Err(err)),
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn zip_allow_right_none<P, F1, F2, T1, T2>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<(T1, Option<T2>), P::Err>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, P::Err>) + 'static,
    F2: Fn(P) -> (P, Result<T2, P::Err>) + 'static,
    P::Err: NotFoundErr,
    P: ParserSource + Undo<T1> + 'static,
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
                            (char_reader.undo(r1), Err(err))
                        }
                    }
                }
            }
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn or<P, F1, F2, R>(first: F1, second: F2) -> Box<dyn Fn(P) -> (P, Result<R, P::Err>)>
where
    R: 'static,
    F1: Fn(P) -> (P, Result<R, P::Err>) + 'static,
    F2: Fn(P) -> (P, Result<R, P::Err>) + 'static,
    P::Err: NotFoundErr,
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

pub fn or_vec<P, T, E, F>(sources: Vec<F>) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
where
    P: ParserSource + 'static,
    T: 'static,
    E: NotFoundErr + 'static,
    F: Fn(P) -> (P, Result<T, E>) + 'static,
{
    Box::new(move |reader| {
        let mut r = reader;
        for source in sources.iter() {
            let x = source(r);
            r = x.0;
            match x.1 {
                Ok(x) => return (r, Ok(x)),
                Err(err) => {
                    if !err.is_not_found_err() {
                        return (r, Err(err));
                    }
                }
            }
        }
        (r, Err(E::not_found_err()))
    })
}

pub fn apply<P, S, M, R, U, E>(mapper: M, source: S) -> Box<dyn Fn(P) -> (P, Result<U, E>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<R, E>) + 'static,
    M: Fn(R) -> U + 'static,
    R: 'static,
    U: 'static,
    E: 'static,
{
    switch(move |x| Ok(mapper(x)), source)
}

pub fn switch<P, S, M, R, U, E>(mapper: M, source: S) -> Box<dyn Fn(P) -> (P, Result<U, E>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<R, E>) + 'static,
    M: Fn(R) -> Result<U, E> + 'static,
    R: 'static,
    U: 'static,
    E: 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => (char_reader, mapper(ch)),
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn take_vec_while<P, FP>(predicate: FP) -> Box<dyn Fn(P) -> (P, Result<Vec<P::Item>, P::Err>)>
where
    FP: Fn(&P::Item) -> bool + 'static,
    P::Err: NotFoundErr,
    P: ParserSource + 'static,
{
    Box::new(move |char_reader| {
        let mut result: Vec<P::Item> = vec![];
        let mut cr: P = char_reader;
        loop {
            let (x, next) = cr.read();
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
                    if predicate(&ch) {
                        result.push(ch);
                    } else {
                        cr = cr.undo(ch);
                        break;
                    }
                }
            }
        }
        if result.is_empty() {
            (cr, Err(P::Err::not_found_err()))
        } else {
            (cr, Ok(result))
        }
    })
}

pub fn take_str_while<P, FP>(predicate: FP) -> Box<dyn Fn(P) -> (P, Result<String, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,
    FP: Fn(char) -> bool + 'static,
    P::Err: NotFoundErr,
{
    Box::new(move |char_reader| {
        let mut result: String = String::new();
        let mut cr: P = char_reader;
        loop {
            let (x, next) = cr.read();
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
                    if predicate(ch) {
                        result.push(ch);
                    } else {
                        cr = cr.undo(ch);
                        break;
                    }
                }
            }
        }
        if result.is_empty() {
            (cr, Err(P::Err::not_found_err()))
        } else {
            (cr, Ok(result))
        }
    })
}

pub fn take_whitespace<P>() -> Box<dyn Fn(P) -> (P, Result<String, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,
    P::Err: NotFoundErr,
{
    take_str_while(|ch| ch == ' ' || ch == '\t')
}

pub fn take_symbol<P>() -> Box<dyn Fn(P) -> (P, Result<char, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,

    P::Err: NotFoundErr,
{
    take_char_if(|ch| {
        (ch > ' ' && ch < '0')
            || (ch > '9' && ch < 'A')
            || (ch > 'Z' && ch < 'a')
            || (ch > 'z' && ch <= '~')
    })
}

pub fn take_identifier<P>() -> Box<dyn Fn(P) -> (P, Result<String, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,

    P::Err: NotFoundErr,
{
    take_str_while(|ch| {
        (ch >= 'a' && ch <= 'z')
            || (ch >= 'A' && ch <= 'Z')
            || (ch >= '0' && ch <= '9')
            || (ch == '.')
    })
}

pub fn switch_from_str<P, S, U, E>(source: S) -> Box<dyn Fn(P) -> (P, Result<(U, String), E>)>
where
    P: ParserSource + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<String, E>) + 'static,
    U: FromStr + 'static,
    E: NotFoundErr + 'static,
{
    Box::new(move |reader| {
        let (reader, next) = source(reader);
        match next {
            Ok(s) => match U::from_str(&s) {
                Ok(u) => (reader, Ok((u, s))),
                Err(_) => (reader.undo(s), Err(E::not_found_err())),
            },
            Err(err) => (reader, Err(err)),
        }
    })
}

pub fn take_any_keyword<P>() -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), P::Err>)>
where
    P: ParserSource<Item = char> + Undo<String> + 'static,
    P::Err: NotFoundErr,
{
    switch_from_str(take_identifier())
}

pub fn take_keyword<P>(needle: Keyword) -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), P::Err>)>
where
    P::Err: NotFoundErr,
    P: ParserSource<Item = char> + Undo<String> + Undo<(Keyword, String)> + 'static,
{
    filter(move |(k, _)| *k == needle, take_any_keyword())
}

pub fn with_pos<P, S, T, E>(source: S) -> Box<dyn Fn(P) -> (P, Result<Locatable<T>, E>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
{
    Box::new(move |char_reader| {
        let pos = char_reader.pos();
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => (char_reader, Ok(ch.at(pos))),
            Err(err) => (char_reader, Err(err)),
        }
    })
}

pub fn with_err_pos<P, S, T, E>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, ErrorEnvelope<E>>)>
where
    P: ParserSource + HasLocation + 'static,

    S: Fn(P) -> (P, Result<T, E>) + 'static,
{
    Box::new(move |char_reader| {
        let pos = char_reader.pos();
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => (char_reader, Ok(ch)),
            Err(err) => (char_reader, Err(err).with_err_at(pos)),
        }
    })
}

pub fn take_lexeme_eol<P>() -> Box<dyn Fn(P) -> (P, Result<Lexeme, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,
    P::Err: NotFoundErr,
{
    apply(
        |x| Lexeme::EOL(x),
        take_str_while(|x| x == '\r' || x == '\n'),
    )
}

pub fn take_lexeme_keyword<P>() -> Box<dyn Fn(P) -> (P, Result<Lexeme, P::Err>)>
where
    P: ParserSource<Item = char> + Undo<String> + 'static,

    P::Err: NotFoundErr,
{
    apply(|(k, s)| Lexeme::Keyword(k, s), take_any_keyword())
}

pub fn take_lexeme_word<P>() -> Box<dyn Fn(P) -> (P, Result<Lexeme, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,
    P::Err: NotFoundErr,
{
    apply(|x| Lexeme::Word(x), take_identifier())
}

pub fn take_lexeme_whitespace<P>() -> Box<dyn Fn(P) -> (P, Result<Lexeme, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,

    P::Err: NotFoundErr,
{
    apply(|x| Lexeme::Whitespace(x), take_whitespace())
}

pub fn take_lexeme_symbol<P>() -> Box<dyn Fn(P) -> (P, Result<Lexeme, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,
    P::Err: NotFoundErr,
{
    apply(|x| Lexeme::Symbol(x), take_symbol())
}

pub fn take_lexeme_digits<P>() -> Box<dyn Fn(P) -> (P, Result<Lexeme, P::Err>)>
where
    P: ParserSource<Item = char> + 'static,
    P::Err: NotFoundErr,
{
    apply(
        |x| Lexeme::Digits(x),
        take_str_while(|ch| ch >= '0' && ch <= '9'),
    )
}

pub fn take_lexeme<P>() -> Box<dyn Fn(P) -> (P, Result<Lexeme, P::Err>)>
where
    P::Err: NotFoundErr,
    P: ParserSource<Item = char> + Undo<String> + 'static,
{
    or(
        take_lexeme_eol(),
        or(
            take_lexeme_keyword(),
            or(
                take_lexeme_word(),
                or(
                    take_lexeme_whitespace(),
                    or(take_lexeme_symbol(), take_lexeme_digits()),
                ),
            ),
        ),
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

impl<T: BufRead + 'static> ParserSource for EolReader<T> {
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
                take_char('\n'),
                apply(
                    // Tradeoff: CRLF becomes just CR
                    // Alternatives:
                    // - Return a String instead of a char
                    // - Return a new enum type instead of a char
                    // - Encode CRLF as a special char e.g. CR = 13 + LF = 10 -> CRLF = 23
                    |(cr, _)| cr,
                    zip_allow_right_none(take_char('\r'), take_char('\n')),
                ),
            ),
            take_any(),
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
                char_reader = char_reader.undo(x);
            }
            _ => {
                pos = Location::new(pos.row(), pos.col() - 1);
                char_reader = char_reader.undo(x);
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

impl<T: BufRead + 'static> Undo<(Keyword, String)> for EolReader<T> {
    fn undo(self, s: (Keyword, String)) -> Self {
        self.undo(s.1)
    }
}

impl<T: BufRead + 'static> Undo<Lexeme> for EolReader<T> {
    fn undo(self, l: Lexeme) -> Self {
        match l {
            Lexeme::EOL(_) => self.undo('\r'),
            Lexeme::Keyword(_, s) | Lexeme::Word(s) | Lexeme::Whitespace(s) | Lexeme::Digits(s) => {
                self.undo(s)
            }
            Lexeme::Symbol(ch) => self.undo(ch),
        }
    }
}

impl<T: BufRead, R> Undo<Locatable<R>> for EolReader<T>
where
    EolReader<T>: Undo<R>,
{
    fn undo(self, x: Locatable<R>) -> Self {
        let Locatable { element, .. } = x;
        self.undo(element)
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
    use crate::common::AtRowCol;

    #[test]
    fn test_new_style() {
        let input = "  hello";
        let char_reader = CharReader::from(input);
        let (char_reader, s) = take_whitespace()(char_reader);
        assert_eq!(s.unwrap(), "  ");
        assert_eq!(take_any()(char_reader).1.unwrap(), 'h');
    }

    #[test]
    fn test2_new_style() {
        let input = "hello";
        let char_reader = CharReader::from(input);
        let (char_reader, x) = or(
            and(take_char('h'), take_char('i')),
            and(take_char('h'), take_char('g')),
        )(char_reader);
        assert_eq!(x.unwrap_err().is_not_found_err(), true);
        assert_eq!(take_any()(char_reader).1.unwrap(), 'h');
    }

    #[test]
    fn test_new_style_location_mapping() {
        let input = "hey\r\nhi\r";
        let char_reader = CharReader::from(input);

        let eol_reader = EolReader::new(char_reader);
        assert_eq!(eol_reader.pos(), Location::start());

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), 'h');
        assert_eq!(eol_reader.pos(), Location::new(1, 2));

        let eol_reader = eol_reader.undo('h');
        assert_eq!(eol_reader.pos(), Location::start());

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), 'h');
        assert_eq!(eol_reader.pos(), Location::new(1, 2));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), 'e');
        assert_eq!(eol_reader.pos(), Location::new(1, 3));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), 'y');
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let eol_reader = eol_reader.undo('\r');
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), 'h');
        assert_eq!(eol_reader.pos(), Location::new(2, 2));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), 'i');
        assert_eq!(eol_reader.pos(), Location::new(2, 3));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(3, 1));
    }

    #[test]
    fn test_new_style_identifier() {
        let input = "hey\r\nhi\r";
        let char_reader = CharReader::from(input);

        let eol_reader = EolReader::new(char_reader);

        let (eol_reader, next) = take_identifier()(eol_reader);
        assert_eq!(next.unwrap(), "hey");
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let (eol_reader, next) = take_identifier()(eol_reader);
        assert_eq!(next.unwrap(), "hi");
        assert_eq!(eol_reader.pos(), Location::new(2, 3));
    }

    #[test]
    fn test_new_style_identifier_with_pos() {
        let input = "hey\r\nhi\r";
        let char_reader = CharReader::from(input);

        let eol_reader = EolReader::new(char_reader);

        let (eol_reader, next) = with_pos(take_identifier())(eol_reader);
        assert_eq!(next.unwrap(), "hey".to_string().at_rc(1, 1));
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let (eol_reader, next) = with_pos(take_identifier())(eol_reader);
        assert_eq!(next.unwrap(), "hi".to_string().at_rc(2, 1));
        assert_eq!(eol_reader.pos(), Location::new(2, 3));
    }

    #[test]
    fn test_new_style_keyword_with_pos() {
        let input = "DIM X";
        let char_reader = CharReader::from(input);

        let eol_reader = EolReader::new(char_reader);

        let (_, result) = apply(
            |(l, (_, r))| (l, r),
            and(
                with_pos(take_any_keyword()),
                and(take_whitespace(), with_pos(take_identifier())),
            ),
        )(eol_reader);
        let (l, r) = result.unwrap();
        assert_eq!(l, (Keyword::Dim, "DIM".to_string()).at_rc(1, 1));
        assert_eq!(r, "X".to_string().at_rc(1, 5));
    }

    #[test]
    fn test_new_style_rollback() {
        let input = "X = 42";
        let char_reader = CharReader::from(input);
        let eol_reader = EolReader::new(char_reader);
        let (_, result) = or(
            and(
                take_identifier(),
                and(take_whitespace(), take_str_while(|ch| ch == '.')),
            ),
            and(
                take_identifier(),
                and(take_whitespace(), take_str_while(|ch| ch == '=')),
            ),
        )(eol_reader);
        let (l, (_, r)) = result.unwrap();
        assert_eq!(l, "X".to_string());
        assert_eq!(r, "=".to_string());
    }

    #[test]
    fn test_eof_is_twice() {
        let mut reader: CharReader<BufReader<Cursor<&str>>> = "123".into();
        assert_eq!(reader.read_ng().unwrap().unwrap(), '1');
        assert_eq!(reader.read_ng().unwrap().unwrap(), '2');
        assert_eq!(reader.read_ng().unwrap().unwrap(), '3');
        assert_eq!(reader.read_ng().unwrap(), None);
        assert_eq!(reader.read_ng().unwrap(), None);
    }
}
