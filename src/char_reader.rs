use crate::common::{
    AtLocation, HasLocation, Locatable, Location, PeekOptCopy, QError, QErrorNode, ReadOpt,
    ToLocatableError,
};
use crate::lexer::{Keyword, Lexeme};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::str::FromStr;

pub trait IsNotFoundErr {
    fn is_not_found_err(&self) -> bool;
}

//
// NotFoundErr
//

pub trait NotFoundErr: IsNotFoundErr {
    fn not_found_err() -> Self;
}

impl IsNotFoundErr for QError {
    fn is_not_found_err(&self) -> bool {
        *self == QError::CannotParse
    }
}

impl IsNotFoundErr for QErrorNode {
    fn is_not_found_err(&self) -> bool {
        self.as_ref().is_not_found_err()
    }
}

impl NotFoundErr for QError {
    fn not_found_err() -> Self {
        QError::CannotParse
    }
}

impl NotFoundErr for QErrorNode {
    fn not_found_err() -> Self {
        QErrorNode::NoPos(QError::CannotParse)
    }
}

//
// ParserSource
//

pub trait ParserSource: Sized {
    fn read(self) -> (Self, Result<char, QErrorNode>);

    fn undo_item(self, item: char) -> Self;
}

fn wrap_err<P, T>(p: P, err: QError) -> (P, Result<T, QErrorNode>)
where
    P: ParserSource + HasLocation,
{
    let pos = p.pos();
    (p, Err(err).with_err_at(pos))
}

//
// Undo
//

pub trait Undo<T> {
    fn undo(self, item: T) -> Self;
}

impl<P: ParserSource> Undo<char> for P {
    fn undo(self, item: char) -> Self {
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
    fn read(self) -> (Self, Result<char, QErrorNode>) {
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
                    Err(QErrorNode::not_found_err()),
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
                                Err(QErrorNode::not_found_err()),
                            )
                        }
                    }
                    Err(err) => (
                        Self {
                            reader,
                            buffer,
                            read_eof,
                        },
                        Err(QErrorNode::NoPos(err.into())),
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

    fn read_dp(&mut self) -> Result<Option<char>, QErrorNode> {
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
    fn peek_copy_dp(&mut self) -> Result<Option<char>, QErrorNode> {
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
// Naming conventions
//
// read_any  : reads the next item. Err::NotFound is okay.
// read_some : reads the next item. Err::NotFound is mapped to the given error.

//
// Parser combinators
//

pub fn read_any<P>() -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    Box::new(|char_reader| char_reader.read())
}

pub fn read_some<P, FE>(err_fn: FE) -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + 'static,
    FE: Fn() -> QErrorNode + 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, result) = char_reader.read();
        match result {
            Err(err) => {
                if err.is_not_found_err() {
                    (char_reader, Err(err_fn()))
                } else {
                    (char_reader, Err(err))
                }
            }
            _ => (char_reader, result),
        }
    })
}

pub fn filter_any<P, T, E, S, F>(source: S, predicate: F) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
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

pub fn filter_some<P, S, F, T, E, FE>(
    source: S,
    predicate: F,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
    F: Fn(&T) -> bool + 'static,
    FE: Fn() -> E + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |reader| {
        let (reader, result) = source(reader);
        match result {
            Ok(ch) => {
                if predicate(&ch) {
                    (reader, Ok(ch))
                } else {
                    (reader, Err(err_fn()))
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    (reader, Err(err_fn()))
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

pub fn filter_copy_any<P, S, F, T, E>(
    source: S,
    predicate: F,
) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
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

pub fn filter_copy_some<P, S, F, T, FE>(
    source: S,
    predicate: F,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
    F: Fn(T) -> bool + 'static,
    T: Copy + 'static,
    FE: Fn() -> QError + 'static,
{
    Box::new(move |reader| {
        let (reader, result) = source(reader);
        match result {
            Ok(ch) => {
                if predicate(ch) {
                    (reader, Ok(ch))
                } else {
                    wrap_err(reader, err_fn())
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    wrap_err(reader, err_fn())
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

pub fn read_any_char_if<P, F>(predicate: F) -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + 'static,
    F: Fn(char) -> bool + 'static,
{
    filter_copy_any(read_any(), predicate)
}

pub fn read_some_char_that<P, F, FE>(
    predicate: F,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    F: Fn(char) -> bool + 'static,
    FE: Fn() -> QError + 'static,
{
    filter_copy_some(read_any(), predicate, err_fn)
}

pub fn try_read_char<P>(needle: char) -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_char_if(move |ch| ch == needle)
}

pub fn demand_char<P, FE>(
    needle: char,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    FE: Fn() -> QError + 'static,
{
    read_some_char_that(move |ch| ch == needle, err_fn)
}

pub fn skip_while<P, FP>(predicate: FP) -> Box<dyn Fn(P) -> (P, Result<(), QErrorNode>)>
where
    P: ParserSource + 'static,
    FP: Fn(char) -> bool + 'static,
{
    Box::new(move |char_reader| {
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
                    if !predicate(ch) {
                        cr = cr.undo(ch);
                        break;
                    }
                }
            }
        }
        (cr, Ok(()))
    })
}

pub fn read_any_str_while<P, FP>(predicate: FP) -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
    FP: Fn(char) -> bool + 'static,
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
            (cr, Err(QErrorNode::not_found_err()))
        } else {
            (cr, Ok(result))
        }
    })
}

pub fn read_some_str_while<P, FP, FE>(
    predicate: FP,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    FP: Fn(char) -> bool + 'static,
    FE: Fn() -> QError + 'static,
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
                        if !result.is_empty() {
                            // undo the char only if we've read at least something successfully
                            // if we haven't it means we shouldn't undo it because the
                            // premise that we're supposed to read at least one char
                            // has not been fulfilled
                            cr = cr.undo(ch);
                        }
                        break;
                    }
                }
            }
        }
        if result.is_empty() {
            wrap_err(cr, err_fn())
        } else {
            (cr, Ok(result))
        }
    })
}

pub fn read_any_whitespace<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_str_while(|ch| ch == ' ' || ch == '\t')
}

pub fn skip_whitespace_ng<P>() -> Box<dyn Fn(P) -> (P, Result<(), QErrorNode>)>
where
    P: ParserSource + 'static,
{
    skip_while(|ch| ch == ' ' || ch == '\t')
}

pub fn skipping_whitespace_ng<P, S, T>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
    T: 'static,
{
    skip_first(skip_whitespace_ng(), source)
}

pub fn read_some_whitespace<P, FE>(err_fn: FE) -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    FE: Fn() -> QError + 'static,
{
    read_some_str_while(|ch| ch == ' ' || ch == '\t', err_fn)
}

pub fn read_any_symbol<P>() -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_char_if(|ch| {
        (ch > ' ' && ch < '0')
            || (ch > '9' && ch < 'A')
            || (ch > 'Z' && ch < 'a')
            || (ch > 'z' && ch <= '~')
    })
}

pub fn read_any_letter<P>() -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_char_if(|ch| (ch >= 'a' && ch <= 'z') || (ch >= 'A' || ch <= 'Z'))
}

pub fn read_some_letter<P, FE>(err_fn: FE) -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    FE: Fn() -> QError + 'static,
{
    read_some_char_that(
        |ch| (ch >= 'a' && ch <= 'z') || (ch >= 'A' || ch <= 'Z'),
        err_fn,
    )
}

/// Reads any identifier. Note that the result might be a keyword.
/// An identifier must start with a letter and consists of letters, numbers and the dot.
pub fn read_any_identifier<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    map_ng(
        if_first_maybe_second(
            read_any_letter(),
            read_any_str_while(|ch| {
                (ch >= 'a' && ch <= 'z')
                    || (ch >= 'A' && ch <= 'Z')
                    || (ch >= '0' && ch <= '9')
                    || (ch == '.')
            }),
        ),
        |(l, opt_r)| {
            let mut result: String = String::new();
            result.push(l);
            if opt_r.is_some() {
                result.push_str(opt_r.unwrap().as_ref());
            }
            result
        },
    )
}

/// Reads any keyword.
pub fn read_any_keyword<P>() -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QErrorNode>)>
where
    P: ParserSource + Undo<String> + 'static,
{
    switch_from_str(read_any_identifier())
}

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn read_any_word<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
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
) -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QErrorNode>)>
where
    P: ParserSource + Undo<String> + Undo<(Keyword, String)> + 'static,
    F: Fn(Keyword) -> bool + 'static,
{
    filter_any(read_any_keyword(), move |(k, _)| predicate(*k))
}

pub fn try_read_keyword<P>(
    needle: Keyword,
) -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QErrorNode>)>
where
    P: ParserSource + Undo<String> + Undo<(Keyword, String)> + 'static,
{
    read_keyword_if(move |k| k == needle)
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

pub fn read_any_eol<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_str_while(|x| x == '\r' || x == '\n')
}

pub fn read_any_digits<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_str_while(|ch| ch >= '0' && ch <= '9')
}

//
// Combine two or more parsers
//

pub fn and_ng<P, F1, F2, T1, T2, E>(
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

pub fn maybe_first_and_second<P, F1, F2, T1, T2, E>(
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

pub fn if_first_demand_second<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, QErrorNode>) + 'static,
    F2: Fn(P) -> (P, Result<T2, QErrorNode>) + 'static,
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
                            wrap_err(char_reader, err_fn())
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

pub fn and_skip_first<P, F1, F2, T1, T2, E>(
    first: F1,
    second: F2,
) -> Box<dyn Fn(P) -> (P, Result<T2, E>)>
where
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, E>) + 'static,
    F2: Fn(P) -> (P, Result<T2, E>) + 'static,
    P: ParserSource + Undo<T1> + 'static,
    E: IsNotFoundErr + 'static,
{
    map_ng(and_ng(first, second), |(_, r)| r)
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

pub fn or_ng<P, F1, F2, T, E>(first: F1, second: F2) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
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

pub fn or_vec_ng<P, T, E, F>(mut sources: Vec<F>) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
where
    P: ParserSource + 'static,
    T: 'static,
    E: IsNotFoundErr + 'static,
    F: Fn(P) -> (P, Result<T, E>) + 'static,
{
    if sources.len() > 2 {
        let first = sources.remove(0);
        or_ng(first, or_vec_ng(sources))
    } else if sources.len() == 2 {
        let second = sources.pop().unwrap();
        let first = sources.pop().unwrap();
        or_ng(first, second)
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
    P: ParserSource + 'static,
    E: IsNotFoundErr,
{
    Box::new(move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(_) => second(char_reader),
            Err(err) => {
                if err.is_not_found_err() {
                    second(char_reader)
                } else {
                    (char_reader, Err(err))
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
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    F1: Fn(P) -> (P, Result<T1, QErrorNode>) + 'static,
    F2: Fn(P) -> (P, Result<T2, QErrorNode>) + 'static,
    T1: 'static,
    T2: 'static,
    FE: Fn() -> QError + 'static,
{
    map_ng(
        if_first_demand_second(first, and_no_undo(read_any_whitespace(), second), err_fn),
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
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QErrorNode>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    F1: Fn(P) -> (P, Result<T1, QErrorNode>) + 'static,
    F2: Fn(P) -> (P, Result<T2, QErrorNode>) + 'static,
    T1: 'static,
    T2: 'static,
    FE: Fn() -> QError + 'static,
{
    map_ng(
        and_ng(
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
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    F1: Fn(P) -> (P, Result<T1, QErrorNode>) + 'static,
    F2: Fn(P) -> (P, Result<T2, QErrorNode>) + 'static,
    T1: 'static,
    T2: 'static,
    FE: Fn() -> QError + 'static,
{
    if_first_demand_second(first, skipping_whitespace_ng(second), err_fn)
}

//
// Modify the result of a parser
//

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

pub fn map_ng<P, S, M, R, U, E>(source: S, mapper: M) -> Box<dyn Fn(P) -> (P, Result<U, E>)>
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

pub fn demand_ng<P, S, M, R, E>(source: S, err_fn: M) -> Box<dyn Fn(P) -> (P, Result<R, E>)>
where
    P: ParserSource + 'static,
    S: Fn(P) -> (P, Result<R, E>) + 'static,
    M: Fn() -> E + 'static,
    R: 'static,
    E: IsNotFoundErr + 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => (char_reader, Ok(ch)),
            Err(err) => {
                if err.is_not_found_err() {
                    (char_reader, Err(err_fn()))
                } else {
                    (char_reader, Err(err))
                }
            }
        }
    })
}

//
// Take multiple items
//

pub fn take_zero_or_more<P, S, T>(source: S) -> Box<dyn Fn(P) -> (P, Result<Vec<T>, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
{
    take_one_or_more(source, QError::not_found_err)
}

pub fn take_one_or_more<P, S, T, FE>(
    source: S,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<Vec<T>, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
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
                    result.push(ch);
                }
            }
        }
        if result.is_empty() {
            wrap_err(cr, err_fn())
        } else {
            (cr, Ok(result))
        }
    })
}

pub fn csv_one_or_more<P, S, R, FE>(
    source: S,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<R, QErrorNode>) + 'static,
    R: 'static,
    FE: Fn() -> QError + 'static,
{
    csv_ensure_no_trailing_comma(take_one_or_more(
        if_first_maybe_second(
            skipping_whitespace_ng(source),
            skipping_whitespace_ng(with_pos(try_read_char(','))),
        ),
        err_fn,
    ))
}

pub fn csv_zero_or_more<P, S, R>(source: S) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<R, QErrorNode>) + 'static,
    R: 'static,
{
    csv_ensure_no_trailing_comma(take_zero_or_more(if_first_maybe_second(
        skipping_whitespace_ng(source),
        skipping_whitespace_ng(with_pos(try_read_char(','))),
    )))
}

fn csv_ensure_no_trailing_comma<P, R, S>(
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<Vec<(R, Option<Locatable<char>>)>, QErrorNode>) + 'static,
    R: 'static,
{
    map_to_result_no_undo(source, |tuples| {
        let last_item = tuples.last().unwrap();
        match last_item {
            (_, Some(trailing_comma)) => {
                Err(QError::SyntaxError("Trailing comma".to_string())).with_err_at(trailing_comma)
            }
            _ => Ok(tuples.into_iter().map(|x| x.0).collect()),
        }
    })
}

pub fn in_parenthesis<P, T, S>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
    T: 'static,
{
    map_to_result_no_undo(
        and_ng(
            try_read_char('('),
            maybe_first_and_second(
                source,
                demand_char(')', || {
                    QError::SyntaxError("Expected closing parenthesis".to_string())
                }),
            ),
        ),
        |(_, (r, _))| match r {
            Some(x) => Ok(x),
            None => Err(QErrorNode::not_found_err()),
        },
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

    pub fn err<R>(self, err: QError) -> (Self, Result<R, QErrorNode>) {
        let pos: Location = self.pos;
        (self, Err(err).with_err_at(pos))
    }

    pub fn undo_and_err_not_found<R, U>(self, item: U) -> (Self, Result<R, QErrorNode>)
    where
        Self: Undo<U>,
    {
        self.undo(item).err(QError::not_found_err())
    }
}

impl<T: BufRead + 'static> ParserSource for EolReader<T> {
    fn read(self) -> (Self, Result<char, QErrorNode>) {
        let Self {
            char_reader,
            mut pos,
            mut line_lengths,
        } = self;
        let (char_reader, next) = or_ng(
            or_ng(
                try_read_char('\n'),
                map_ng(
                    // Tradeoff: CRLF becomes just CR
                    // Alternatives:
                    // - Return a String instead of a char
                    // - Return a new enum type instead of a char
                    // - Encode CRLF as a special char e.g. CR = 13 + LF = 10 -> CRLF = 23
                    if_first_maybe_second(try_read_char('\r'), try_read_char('\n')),
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
        let (char_reader, s) = read_any_whitespace()(char_reader);
        assert_eq!(s.unwrap(), "  ");
        assert_eq!(read_any()(char_reader).1.unwrap(), 'h');
    }

    #[test]
    fn test2_new_style() {
        let input = "hello";
        let char_reader = CharReader::from(input);
        let (char_reader, x) = or_ng(
            and_ng(try_read_char('h'), try_read_char('i')),
            and_ng(try_read_char('h'), try_read_char('g')),
        )(char_reader);
        assert_eq!(x.unwrap_err().is_not_found_err(), true);
        assert_eq!(read_any()(char_reader).1.unwrap(), 'h');
    }

    #[test]
    fn test_new_style_location_mapping() {
        let input = "hey\r\nhi\r";
        let char_reader = CharReader::from(input);

        let eol_reader = EolReader::new(char_reader);
        assert_eq!(eol_reader.pos(), Location::start());

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), 'h');
        assert_eq!(eol_reader.pos(), Location::new(1, 2));

        let eol_reader = eol_reader.undo('h');
        assert_eq!(eol_reader.pos(), Location::start());

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), 'h');
        assert_eq!(eol_reader.pos(), Location::new(1, 2));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), 'e');
        assert_eq!(eol_reader.pos(), Location::new(1, 3));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), 'y');
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let eol_reader = eol_reader.undo('\r');
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), 'h');
        assert_eq!(eol_reader.pos(), Location::new(2, 2));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), 'i');
        assert_eq!(eol_reader.pos(), Location::new(2, 3));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(3, 1));
    }

    #[test]
    fn test_new_style_identifier() {
        let input = "hey\r\nhi\r";
        let char_reader = CharReader::from(input);

        let eol_reader = EolReader::new(char_reader);

        let (eol_reader, next) = read_any_identifier()(eol_reader);
        assert_eq!(next.unwrap(), "hey");
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let (eol_reader, next) = read_any_identifier()(eol_reader);
        assert_eq!(next.unwrap(), "hi");
        assert_eq!(eol_reader.pos(), Location::new(2, 3));
    }

    #[test]
    fn test_new_style_identifier_with_pos() {
        let input = "hey\r\nhi\r";
        let char_reader = CharReader::from(input);

        let eol_reader = EolReader::new(char_reader);

        let (eol_reader, next) = with_pos(read_any_identifier())(eol_reader);
        assert_eq!(next.unwrap(), "hey".to_string().at_rc(1, 1));
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = read_any()(eol_reader);
        assert_eq!(next.unwrap(), '\r');
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let (eol_reader, next) = with_pos(read_any_identifier())(eol_reader);
        assert_eq!(next.unwrap(), "hi".to_string().at_rc(2, 1));
        assert_eq!(eol_reader.pos(), Location::new(2, 3));
    }

    #[test]
    fn test_new_style_keyword_with_pos() {
        let input = "DIM X";
        let char_reader = CharReader::from(input);

        let eol_reader = EolReader::new(char_reader);

        let (_, result) = map_ng(
            and_ng(
                with_pos(read_any_keyword()),
                and_ng(read_any_whitespace(), with_pos(read_any_identifier())),
            ),
            |(l, (_, r))| (l, r),
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
        let (_, result) = or_ng(
            and_ng(
                read_any_identifier(),
                and_ng(read_any_whitespace(), read_any_str_while(|ch| ch == '.')),
            ),
            and_ng(
                read_any_identifier(),
                and_ng(read_any_whitespace(), read_any_str_while(|ch| ch == '=')),
            ),
        )(eol_reader);
        let (l, (_, r)) = result.unwrap();
        assert_eq!(l, "X".to_string());
        assert_eq!(r, "=".to_string());
    }

    #[test]
    fn test_eof_is_twice() {
        let mut reader: CharReader<BufReader<Cursor<&str>>> = "123".into();
        assert_eq!(reader.read_dp().unwrap().unwrap(), '1');
        assert_eq!(reader.read_dp().unwrap().unwrap(), '2');
        assert_eq!(reader.read_dp().unwrap().unwrap(), '3');
        assert_eq!(reader.read_dp().unwrap(), None);
        assert_eq!(reader.read_dp().unwrap(), None);
    }
}
