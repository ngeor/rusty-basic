use crate::common::{
    AtLocation, HasLocation, Locatable, Location, PeekOptCopy, QError, QErrorNode, ReadOpt,
};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::str::FromStr;

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

impl NotFoundErr for QErrorNode {
    fn is_not_found_err(&self) -> bool {
        let err: &QError = self.as_ref();
        err.is_not_found_err()
    }

    fn not_found_err() -> Self {
        QErrorNode::NoPos(QError::not_found_err())
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

pub type CharReaderResult = Result<char, QErrorNode>;

pub type CRR<T> = (CharReader<T>, CharReaderResult);

pub trait ParserSource: Sized {
    type Item;
    type Err;

    fn read(self) -> (Self, Result<Self::Item, Self::Err>);

    fn undo_mine(self, item: Self::Item) -> Self;
}

pub trait Undoable<T> {
    fn undo(self, item: T) -> Self;
}

impl<P: ParserSource> Undoable<P::Item> for P {
    fn undo(self, item: P::Item) -> Self {
        self.undo_mine(item)
    }
}

impl<T: BufRead> ParserSource for CharReader<T> {
    type Item = char;
    type Err = QErrorNode;

    fn read(self) -> (Self, CharReaderResult) {
        let Self {
            mut reader,
            mut buffer,
            mut read_eof,
        } = self;
        if buffer.is_empty() {
            if read_eof {
                (
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

    fn undo_mine(self, ch: char) -> Self {
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

pub fn take_any<P: ParserSource>() -> impl Fn(P) -> (P, Result<P::Item, P::Err>) {
    |char_reader| char_reader.read()
}

pub fn take_char<T: BufRead>(needle: char) -> impl Fn(CharReader<T>) -> CRR<T> {
    move |char_reader| {
        let (char_reader, result) = char_reader.read();
        match result {
            Ok(ch) => {
                if ch == needle {
                    (char_reader, Ok(ch))
                } else {
                    (char_reader.undo(ch), Err(QErrorNode::not_found_err()))
                }
            }
            CharReaderResult::Err(err) => (char_reader, CharReaderResult::Err(err)),
        }
    }
}

pub fn and<P: ParserSource, F1, F2, T1, T2>(
    first: F1,
    second: F2,
) -> impl Fn(P) -> (P, Result<(T1, T2), P::Err>)
where
    F1: Fn(P) -> (P, Result<T1, P::Err>),
    F2: Fn(P) -> (P, Result<T2, P::Err>),
    P: Undoable<T1>,
{
    move |char_reader| {
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
    }
}

pub fn zip_allow_right_none<P: ParserSource, F1, F2, T1, T2>(
    first: F1,
    second: F2,
) -> impl Fn(P) -> (P, Result<(T1, Option<T2>), P::Err>)
where
    F1: Fn(P) -> (P, Result<T1, P::Err>),
    F2: Fn(P) -> (P, Result<T2, P::Err>),
    P::Err: NotFoundErr,
    P: Undoable<T1>,
{
    move |char_reader| {
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
    }
}

pub fn or<P: ParserSource, F1, F2, R>(first: F1, second: F2) -> impl Fn(P) -> (P, Result<R, P::Err>)
where
    F1: Fn(P) -> (P, Result<R, P::Err>),
    F2: Fn(P) -> (P, Result<R, P::Err>),
    P::Err: NotFoundErr,
{
    move |char_reader| {
        let (char_reader, res1) = first(char_reader);
        match res1 {
            Ok(ch) => (char_reader, Ok(ch)),
            Err(err) => {
                if err.is_not_found_err() {
                    second(char_reader)
                } else {
                    (char_reader, Err(err))
                }
            }
        }
    }
}

pub fn apply<P: ParserSource, S, M, R, U>(
    mapper: M,
    source: S,
) -> impl Fn(P) -> (P, Result<U, P::Err>)
where
    M: Fn(R) -> U,
    S: Fn(P) -> (P, Result<R, P::Err>),
{
    move |char_reader| {
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => (char_reader, Ok(mapper(ch))),
            Err(err) => (char_reader, Err(err)),
        }
    }
}

pub fn take_eol<T: BufRead>(
) -> impl Fn(CharReader<T>) -> (CharReader<T>, Result<String, QErrorNode>) {
    or(
        apply(
            |x| {
                let mut s = String::new();
                s.push(x);
                s
            },
            take_char('\n'),
        ),
        apply(
            |(l, r)| {
                let mut s = String::new();
                s.push(l);
                if r.is_some() {
                    s.push(r.unwrap())
                }
                s
            },
            zip_allow_right_none(take_char('\r'), take_char('\n')),
        ),
    )
}

pub fn take_while<P: ParserSource, FP>(
    predicate: FP,
) -> impl Fn(P) -> (P, Result<Vec<P::Item>, P::Err>)
where
    FP: Fn(&P::Item) -> bool,
    P::Err: NotFoundErr,
{
    move |char_reader| {
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
    }
}

pub fn take_whitespace<T: BufRead>(
) -> impl Fn(CharReader<T>) -> (CharReader<T>, Result<String, QErrorNode>) {
    apply(
        |v| {
            v.into_iter()
                .fold(String::new(), |acc, c| format!("{}{}", acc, c))
        },
        take_while(|ch| *ch == ' '),
    )
}

// TODO take_whitespace
// TODO with location respecting EOL
// TODO 1. merge back to CharReader 2. return 10+13=23 for CRLF to keep it simpler
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EolReaderResult {
    Some(char),
    CR,
    LF,
    CRLF,
}

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

impl<T: BufRead> ParserSource for EolReader<T> {
    type Item = EolReaderResult;
    type Err = QErrorNode;

    fn read(self) -> (Self, Result<EolReaderResult, QErrorNode>) {
        let Self {
            char_reader,
            mut pos,
            mut line_lengths,
        } = self;
        let (char_reader, next) = or(
            or(
                apply(|_| EolReaderResult::LF, take_char('\n')),
                apply(
                    |(_, r)| {
                        if r.is_some() {
                            EolReaderResult::CRLF
                        } else {
                            EolReaderResult::CR
                        }
                    },
                    zip_allow_right_none(take_char('\r'), take_char('\n')),
                ),
            ),
            apply(|ch| EolReaderResult::Some(ch), take_any()),
        )(char_reader);
        match next {
            Ok(EolReaderResult::CR) | Ok(EolReaderResult::CRLF) | Ok(EolReaderResult::LF) => {
                if line_lengths.len() + 1 == (pos.row() as usize) {
                    line_lengths.push(pos.col());
                }
                pos.inc_row();
            }
            Ok(EolReaderResult::Some(_)) => {
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

    fn undo_mine(self, x: EolReaderResult) -> Self {
        let Self {
            mut char_reader,
            mut pos,
            line_lengths,
        } = self;
        match x {
            EolReaderResult::Some(ch) => {
                pos = Location::new(pos.row(), pos.col() - 1);
                char_reader = char_reader.undo(ch);
            }
            EolReaderResult::CR => {
                pos = Location::new(pos.row() - 1, line_lengths[(pos.row() - 2) as usize]);
                char_reader = char_reader.undo('\r');
            }
            EolReaderResult::LF => {
                pos = Location::new(pos.row() - 1, line_lengths[(pos.row() - 2) as usize]);
                char_reader = char_reader.undo('\n');
            }
            EolReaderResult::CRLF => {
                pos = Location::new(pos.row() - 1, line_lengths[(pos.row() - 2) as usize]);
                char_reader = char_reader.undo('\n');
                char_reader = char_reader.undo('\r');
            }
        }
        Self {
            char_reader,
            pos,
            line_lengths,
        }
    }
}

impl<T: BufRead> Undoable<char> for EolReader<T> {
    fn undo(self, x: char) -> Self {
        self.undo(EolReaderResult::Some(x))
    }
}

impl<T: BufRead> Undoable<String> for EolReader<T> {
    fn undo(self, s: String) -> Self {
        let mut result = self;
        for ch in s.chars().rev() {
            result = result.undo(ch);
        }
        result
    }
}

impl<T: BufRead> Undoable<(crate::lexer::Keyword, String)> for EolReader<T> {
    fn undo(self, s: (crate::lexer::Keyword, String)) -> Self {
        self.undo(s.1)
    }
}

impl<T: BufRead, R> Undoable<Locatable<R>> for EolReader<T>
where
    EolReader<T>: Undoable<R>,
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

pub fn take_whitespace2<T: BufRead>(
) -> impl Fn(EolReader<T>) -> (EolReader<T>, Result<String, QErrorNode>) {
    apply(
        |v| {
            v.into_iter()
                .fold(String::new(), |acc, _| format!("{} ", acc))
        },
        take_while(|x| match x {
            EolReaderResult::Some(ch) => (*ch == ' '),
            _ => false,
        }),
    )
}

pub fn take_identifier<T: BufRead>(
) -> impl Fn(EolReader<T>) -> (EolReader<T>, Result<String, QErrorNode>) {
    apply(
        |v| {
            v.into_iter()
                .map(|x| match x {
                    EolReaderResult::Some(ch) => ch,
                    _ => '\n',
                })
                .fold(String::new(), |acc, c| format!("{}{}", acc, c))
        },
        take_while(|x| match x {
            EolReaderResult::Some(ch) => {
                (*ch >= 'a' && *ch <= 'z')
                    || (*ch >= 'A' && *ch <= 'Z')
                    || (*ch >= '0' && *ch <= '9')
                    || (*ch == '.')
            }
            _ => false,
        }),
    )
}

pub fn take_keyword<T: BufRead>() -> impl Fn(
    EolReader<T>,
) -> (
    EolReader<T>,
    Result<(crate::lexer::Keyword, String), QErrorNode>,
) {
    move |reader| {
        let (mut reader, result) = take_identifier()(reader);
        match result {
            Ok(mut s) => {
                match crate::lexer::Keyword::from_str(&s) {
                    Ok(k) => (reader, Ok((k, s))),
                    Err(_) => {
                        // need to undo all characters of the string
                        // and return not found
                        while !s.is_empty() {
                            reader = reader.undo(EolReaderResult::Some(s.pop().unwrap()));
                        }

                        (reader, Err(QErrorNode::not_found_err()))
                    }
                }
            }
            Err(err) => (reader, Err(err)),
        }
    }
}

pub fn with_pos<P: ParserSource + HasLocation, S, R>(
    source: S,
) -> impl Fn(P) -> (P, Result<Locatable<R>, P::Err>)
where
    S: Fn(P) -> (P, Result<R, P::Err>),
{
    move |char_reader| {
        let pos = char_reader.pos();
        let (char_reader, next) = source(char_reader);
        match next {
            Ok(ch) => (char_reader, Ok(ch.at(pos))),
            Err(err) => (char_reader, Err(err)),
        }
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
        assert_eq!(next.unwrap(), EolReaderResult::Some('h'));
        assert_eq!(eol_reader.pos(), Location::new(1, 2));

        let eol_reader = eol_reader.undo(EolReaderResult::Some('h'));
        assert_eq!(eol_reader.pos(), Location::start());

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), EolReaderResult::Some('h'));
        assert_eq!(eol_reader.pos(), Location::new(1, 2));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), EolReaderResult::Some('e'));
        assert_eq!(eol_reader.pos(), Location::new(1, 3));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), EolReaderResult::Some('y'));
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), EolReaderResult::CRLF);
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let eol_reader = eol_reader.undo(EolReaderResult::CRLF);
        assert_eq!(eol_reader.pos(), Location::new(1, 4));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), EolReaderResult::CRLF);
        assert_eq!(eol_reader.pos(), Location::new(2, 1));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), EolReaderResult::Some('h'));
        assert_eq!(eol_reader.pos(), Location::new(2, 2));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), EolReaderResult::Some('i'));
        assert_eq!(eol_reader.pos(), Location::new(2, 3));

        let (eol_reader, next) = take_any()(eol_reader);
        assert_eq!(next.unwrap(), EolReaderResult::CR);
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
        assert_eq!(next.unwrap(), EolReaderResult::CRLF);
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
        assert_eq!(next.unwrap(), EolReaderResult::CRLF);
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

        // let (eol_reader, x) = with_pos(take_keyword())(eol_reader);
        // assert_eq!(x.unwrap(), crate::lexer::Keyword::Dim.at_rc(1,1));

        let (eol_reader, result) = apply(
            |(l, (_, r))| (l, r),
            and(
                with_pos(take_keyword()),
                and(take_whitespace2(), with_pos(take_identifier())),
            ),
        )(eol_reader);
        let (l, r) = result.unwrap();
        assert_eq!(
            l,
            (crate::lexer::Keyword::Dim, "DIM".to_string()).at_rc(1, 1)
        );
        assert_eq!(r, "X".to_string().at_rc(1, 5));
    }

    #[test]
    fn test_new_style_rollback() {
        let input = "X = 42";
        let char_reader = CharReader::from(input);
        let eol_reader = EolReader::new(char_reader);
        let (eol_reader, result) = or(
            and(
                take_identifier(),
                and(
                    take_whitespace2(),
                    take_while(|ch| *ch == EolReaderResult::Some('.')),
                ),
            ),
            and(
                take_identifier(),
                and(
                    take_whitespace2(),
                    take_while(|ch| *ch == EolReaderResult::Some('=')),
                ),
            ),
        )(eol_reader);
        let (l, (_, r)) = result.unwrap();
        assert_eq!(l, "X".to_string());
        assert_eq!(r, vec![EolReaderResult::Some('=')]);
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
