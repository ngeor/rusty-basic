use crate::common::QError;
use crate::common::{AtLocation, CaseInsensitiveString, ErrorEnvelope, HasLocation, Locatable};
use crate::parser::pc::common::{
    and, demand, drop_left, many_with_terminating_indicator, map_default_to_not_found, negate,
    opt_seq2, seq3,
};
use crate::parser::pc::map::map;
use crate::parser::pc::str::{
    one_or_more_if, str_case_insensitive, zero_or_more_if_leading_remaining,
};
/// Parser combinators specific to this parser (e.g. for keywords)
use crate::parser::pc::{read, read_if, Reader, ReaderResult, Undo};
use crate::parser::types::{Keyword, Name, QualifiedName, TypeQualifier};
use std::convert::TryInto;

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
            Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) => {
                let first = self.undo(qualifier);
                first.undo(bare_name)
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

fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

fn is_letter(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')
}

fn is_non_leading_identifier_without_dot(ch: char) -> bool {
    is_letter(ch) || is_digit(ch)
}

fn is_non_leading_identifier_with_dot(ch: char) -> bool {
    is_non_leading_identifier_without_dot(ch) || (ch == '.')
}

fn is_symbol(ch: char) -> bool {
    (ch > ' ' && ch < '0')
        || (ch > '9' && ch < 'A')
        || (ch > 'Z' && ch < 'a')
        || (ch > 'z' && ch <= '~')
}

pub fn any_symbol<R, E>() -> impl Fn(R) -> ReaderResult<R, char, E>
where
    R: Reader<Item = char, Err = E> + 'static,
{
    read_if(is_symbol)
}

pub fn any_letter<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, char, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
{
    read_if(is_letter)
}

/// Reads any identifier. Note that the result might be a keyword.
/// An identifier must start with a letter and consists of letters, numbers and the dot.
pub fn any_identifier_with_dot<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    E: 'static,
{
    map_default_to_not_found(zero_or_more_if_leading_remaining(
        is_letter,
        is_non_leading_identifier_with_dot,
    ))
}

pub fn any_identifier_without_dot<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    E: 'static,
{
    map_default_to_not_found(zero_or_more_if_leading_remaining(
        is_letter,
        is_non_leading_identifier_without_dot,
    ))
}

pub fn keyword<R, E>(needle: Keyword) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    E: 'static,
{
    map(
        and(
            str_case_insensitive(needle.as_str()), // find keyword
            negate(read_if(is_non_leading_identifier_with_dot)), // ensure it's a whole word
        ),
        move |(s, _)| (needle, s),
    )
}

pub fn demand_keyword<R>(
    needle: Keyword,
) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), QError>>
where
    R: Reader<Item = char, Err = QError> + 'static,
{
    demand(
        keyword(needle),
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

pub fn any_digits<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
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
    many_with_terminating_indicator(opt_seq2(
        source,
        crate::parser::pc::ws::zero_or_more_around(read(',')),
    ))
}

/// Parses opening and closing parenthesis around the given source.
///
/// Panics if the source returns `Ok(None)`.
pub fn in_parenthesis<R, S, T>(source: S) -> Box<dyn Fn(R) -> ReaderResult<R, T, QError>>
where
    R: Reader<Item = char, Err = QError> + 'static,
    S: Fn(R) -> ReaderResult<R, T, QError> + 'static,
    T: 'static,
{
    map(
        seq3(
            read('('),
            source,
            demand(
                read(')'),
                QError::syntax_error_fn("Expected: closing parenthesis"),
            ),
        ),
        |(_, r, _)| r,
    )
}
