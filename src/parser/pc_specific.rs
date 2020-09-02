use crate::common::QError;
use crate::common::{AtLocation, CaseInsensitiveString, ErrorEnvelope, HasLocation, Locatable};
use crate::parser::pc::common::{
    and, demand, drop_left, filter, map_default_to_not_found, opt_seq2, seq3, zero_or_more,
};
use crate::parser::pc::copy::{read_if, try_read};
use crate::parser::pc::map::{map, source_and_then_some};
use crate::parser::pc::str::{one_or_more_if, switch_from_str, zero_or_more_if_leading_remaining};
/// Parser combinators specific to this parser (e.g. for keywords)
use crate::parser::pc::{Reader, ReaderResult, Undo};
use crate::parser::types::{Keyword, Name, TypeQualifier};
use std::convert::TryInto;
use std::str::FromStr;

// ========================================================
// Undo support
// ========================================================

impl<R: Reader<Item = char>> Undo<TypeQualifier> for R {
    fn undo(self, s: TypeQualifier) -> Self {
        let ch: char = s.try_into().unwrap();
        self.undo_item(ch)
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
// when Reader + HasLocation
// ========================================================

/// Creates a function that maps the result of the source into a locatable result,
/// using the position of the reader just before invoking the source.
pub fn with_pos<R, S, T, E>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, Locatable<T>, E>>
where
    R: Reader<Err = E> + HasLocation + 'static,
    S: Fn(R) -> ReaderResult<R, T, E> + 'static,
{
    Box::new(move |reader| {
        // capture pos before invoking source
        let pos = reader.pos();
        source(reader).and_then(|(reader, ok_res)| Ok((reader, ok_res.map(|x| x.at(pos)))))
    })
}

// ========================================================
// Error location
// ========================================================

pub fn with_err_at<R, S, T, E>(
    source: S,
) -> Box<dyn Fn(R) -> Result<(R, Option<T>), ErrorEnvelope<E>>>
where
    R: Reader<Err = E> + HasLocation + 'static,
    S: Fn(R) -> ReaderResult<R, T, E> + 'static,
{
    Box::new(move |reader| {
        let result = source(reader);
        match result {
            Ok(x) => Ok(x),
            Err((reader, err)) => {
                // capture pos after invoking source
                let pos = reader.pos();
                Err(ErrorEnvelope::Pos(err, pos))
            }
        }
    })
}

// ========================================================
// Miscellaneous
// ========================================================

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
    map_default_to_not_found(zero_or_more_if_leading_remaining(
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
    switch_from_str(read_any_identifier())
}

/// Reads any word, i.e. any identifier which is not a keyword.
pub fn read_any_word<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    E: 'static,
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
    E: 'static,
{
    filter(read_any_keyword(), move |(k, _)| predicate(*k))
}

// TODO optimize
pub fn try_read_keyword<R, E>(
    needle: Keyword,
) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    E: 'static,
{
    read_keyword_if(move |k| k == needle)
}

pub fn demand_keyword<R>(
    needle: Keyword,
) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), QError>>
where
    R: Reader<Item = char, Err = QError> + 'static,
{
    demand(
        try_read_keyword(needle),
        QError::syntax_error_fn(format!("Expected: {}", needle)),
    )
}

pub fn demand_guarded_keyword<R>(
    needle: Keyword,
) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), QError>>
where
    R: Reader<Item = char, Err = QError> + 'static,
{
    drop_left(and(
        demand(
            crate::parser::pc::ws::one_or_more(),
            QError::syntax_error_fn(format!("Expected: whitespace before {}", needle)),
        ),
        demand_keyword(needle),
    ))
}

pub fn read_any_digits<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    E: 'static,
{
    one_or_more_if(is_digit)
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
    E: 'static,
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
