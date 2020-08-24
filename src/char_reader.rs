use crate::common::{
    AtLocation, CaseInsensitiveString, ErrorEnvelope, HasLocation, Locatable, Location, QError,
    QErrorNode, ToLocatableError,
};
use crate::lexer::Keyword;
use std::collections::VecDeque;
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

fn is_symbol(ch: char) -> bool {
    (ch > ' ' && ch < '0')
        || (ch > '9' && ch < 'A')
        || (ch > 'Z' && ch < 'a')
        || (ch > 'z' && ch <= '~')
}

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

impl<P: ParserSource> Undo<()> for P {
    fn undo(self, _item: ()) -> Self {
        self
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
) -> Box<dyn Fn(P) -> (P, Result<T, ErrorEnvelope<E>>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, ErrorEnvelope<E>>) + 'static,
    F: Fn(&T) -> bool + 'static,
    FE: Fn() -> E + 'static,
    ErrorEnvelope<E>: IsNotFoundErr,
{
    Box::new(move |reader| {
        let pos = reader.pos();
        let (reader, result) = source(reader);
        match result {
            Ok(ch) => {
                if predicate(&ch) {
                    (reader, Ok(ch))
                } else {
                    (reader, Err(err_fn()).with_err_at(pos))
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    (reader, Err(err_fn()).with_err_at(pos))
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

pub fn undo_if_ok<P, S, T, E>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, E>)>
where
    P: ParserSource + Undo<T> + 'static,
    E: NotFoundErr,
    S: Fn(P) -> (P, Result<T, E>) + 'static,
    T: Copy + 'static,
{
    Box::new(move |reader| {
        let (reader, result) = source(reader);
        match result {
            Ok(ch) => (reader.undo(ch), Ok(ch)),
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

pub fn skip_while<P, FP>(predicate: FP) -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
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
        (cr, Ok(result))
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

pub fn read_any_whitespace<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_str_while(is_whitespace)
}

pub fn skipping_whitespace_around<P, T, S>(
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + Undo<String> + Undo<T> + 'static,
    T: 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
{
    map_ng(
        and_ng(skip_whitespace_ng(), and_ng(source, skip_whitespace_ng())),
        |(_, (l, _))| l,
    )
}

pub fn skip_whitespace_ng<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    skip_while(is_whitespace)
}

pub fn skip_whitespace_eol_ng<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    skip_while(|ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r')
}

pub fn skipping_whitespace_ng<P, S, T>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
    T: 'static,
{
    skip_first(skip_whitespace_ng(), source)
}

pub fn skipping_whitespace_lazy_ng<P, S, T>(
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + Undo<String> + 'static,
    S: Fn() -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)> + 'static,
    T: 'static,
{
    skip_first_lazy(skip_whitespace_ng(), source)
}

pub fn skipping_whitespace_eol_ng<P, S, T>(
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
    T: 'static,
{
    skip_first(skip_whitespace_eol_ng(), source)
}

pub fn read_any_symbol<P>() -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_char_if(is_symbol)
}

pub fn read_any_letter<P>() -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_char_if(is_letter)
}

pub fn read_some_letter<P, FE>(err_fn: FE) -> Box<dyn Fn(P) -> (P, Result<char, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    FE: Fn() -> QError + 'static,
{
    read_some_char_that(is_letter, err_fn)
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

pub fn demand_keyword<P>(
    needle: Keyword,
) -> Box<dyn Fn(P) -> (P, Result<(Keyword, String), QErrorNode>)>
where
    P: ParserSource + Undo<String> + HasLocation + 'static,
{
    filter_some(
        read_any_keyword(),
        move |(k, _)| *k == needle,
        move || QError::SyntaxError(format!("Expected keyword {}", needle)),
    )
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
    read_any_str_while(is_eol)
}

pub fn read_any_eol_whitespace<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_str_while(|x| x == '\r' || x == '\n' || x == ' ' || x == '\t')
}

pub fn read_any_digits<P>() -> Box<dyn Fn(P) -> (P, Result<String, QErrorNode>)>
where
    P: ParserSource + 'static,
{
    read_any_str_while(is_digit)
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

pub fn if_first_demand_second_lazy<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    T1: 'static,
    T2: 'static,
    F1: Fn(P) -> (P, Result<T1, QErrorNode>) + 'static,
    F2: Fn() -> Box<dyn Fn(P) -> (P, Result<T2, QErrorNode>)> + 'static,
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
    map_ng(and_ng(negate(predicate_source), source), |(_, r)| r)
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

pub fn with_some_whitespace_between_lazy<P, F1, F2, T1, T2, FE>(
    first: F1,
    second: F2,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<(T1, T2), QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    F1: Fn(P) -> (P, Result<T1, QErrorNode>) + 'static,
    F2: Fn() -> Box<dyn Fn(P) -> (P, Result<T2, QErrorNode>)> + 'static,
    T1: 'static,
    T2: 'static,
    FE: Fn() -> QError + 'static,
{
    map_ng(
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
    P: ParserSource + HasLocation + Undo<String> + 'static,
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

pub fn map_to_reader_with_err_at_pos<P, S, M, T, U, E>(
    source: S,
    mapper: M,
) -> Box<dyn Fn(P) -> (P, Result<U, ErrorEnvelope<E>>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, ErrorEnvelope<E>>) + 'static,
    M: Fn(P, T) -> (P, Result<U, E>) + 'static,
    T: 'static,
    U: 'static,
    E: 'static,
{
    Box::new(move |char_reader| {
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => {
                let pos = char_reader.pos();
                let (char_reader, res) = mapper(char_reader, ch);
                (char_reader, res.with_err_at(pos))
            }
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

pub fn map_to_result_no_undo_with_err_at_pos<P, S, M, T, U, E>(
    source: S,
    mapper: M,
) -> Box<dyn Fn(P) -> (P, Result<U, ErrorEnvelope<E>>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, ErrorEnvelope<E>>) + 'static,
    M: Fn(T) -> Result<U, E> + 'static,
    T: 'static,
    U: 'static,
    E: 'static,
{
    map_to_reader_with_err_at_pos(source, move |reader, ok| (reader, mapper(ok)))
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

pub fn take_zero_or_more_to_default<P, S, T, F>(
    source: S,
    is_terminal: F,
) -> Box<dyn Fn(P) -> (P, Result<Vec<T>, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
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
) -> Box<dyn Fn(P) -> (P, Result<Vec<T>, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
    T: 'static,
    F: Fn(&T) -> bool + 'static,
{
    take_one_or_more(source, is_terminal, QError::not_found_err)
}

pub fn take_one_or_more<P, S, T, F, FE>(
    source: S,
    is_terminal: F,
    err_fn: FE,
) -> Box<dyn Fn(P) -> (P, Result<Vec<T>, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
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
    P: ParserSource + HasLocation + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<R, QErrorNode>) + 'static,
    R: 'static,
    FE: Fn() -> QError + 'static,
{
    map_ng(
        take_one_or_more(
            if_first_maybe_second(
                skipping_whitespace_ng(source),
                skipping_whitespace_ng(with_pos(try_read_char(','))),
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
) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QErrorNode>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    S: Fn() -> Box<dyn Fn(P) -> (P, Result<R, QErrorNode>)> + 'static,
    R: 'static,
    FE: Fn() -> QError + 'static,
{
    map_ng(
        take_one_or_more(
            if_first_maybe_second(
                skipping_whitespace_lazy_ng(source),
                skipping_whitespace_ng(with_pos(try_read_char(','))),
            ),
            |x| x.1.is_none(),
            err_fn,
        ),
        |x| x.into_iter().map(|x| x.0).collect(),
    )
}

pub fn csv_zero_or_more<P, S, R>(source: S) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QErrorNode>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    S: Fn(P) -> (P, Result<R, QErrorNode>) + 'static,
    R: 'static,
{
    map_ng(
        take_zero_or_more(
            if_first_maybe_second(
                skipping_whitespace_ng(source),
                skipping_whitespace_ng(with_pos(try_read_char(','))),
            ),
            |x| x.1.is_none(),
        ),
        |x| x.into_iter().map(|x| x.0).collect(),
    )
}

pub fn csv_zero_or_more_lazy<P, S, R>(
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<Vec<R>, QErrorNode>)>
where
    P: ParserSource + HasLocation + Undo<String> + 'static,
    S: Fn() -> Box<dyn Fn(P) -> (P, Result<R, QErrorNode>)> + 'static,
    R: 'static,
{
    map_ng(
        take_zero_or_more(
            if_first_maybe_second(
                skipping_whitespace_lazy_ng(source),
                skipping_whitespace_ng(with_pos(try_read_char(','))),
            ),
            |x| x.1.is_none(),
        ),
        |x| x.into_iter().map(|x| x.0).collect(),
    )
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
            maybe_first_and_second_no_undo(
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

pub fn in_parenthesis_lazy<P, T, S>(source: S) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + HasLocation + 'static,
    S: Fn() -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)> + 'static,
    T: 'static,
{
    map_to_result_no_undo(
        and_ng(
            try_read_char('('),
            maybe_first_lazy_and_second_no_undo(
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
// Keyword guards
//

pub fn with_keyword_before<P, T, S>(
    needle: Keyword,
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + HasLocation + Undo<String> + Undo<(Keyword, String)> + 'static,
    T: 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
{
    map_ng(
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
) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + HasLocation + Undo<String> + Undo<(Keyword, String)> + 'static,
    T: 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
    FE: Fn() -> QError + 'static,
{
    // TODO remove the skipping_whitespace_ng , remove with_keyword_after altogether
    map_ng(
        if_first_demand_second(
            source,
            skipping_whitespace_ng(try_read_keyword(needle)),
            err_fn,
        ),
        |(l, _)| l,
    )
}

pub fn with_two_keywords<P, T, S>(
    first: Keyword,
    second: Keyword,
    source: S,
) -> Box<dyn Fn(P) -> (P, Result<T, QErrorNode>)>
where
    P: ParserSource + HasLocation + Undo<String> + Undo<(Keyword, String)> + 'static,
    T: 'static,
    S: Fn(P) -> (P, Result<T, QErrorNode>) + 'static,
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
