/// Parser combinators specific to this project (e.g. for keywords)
use crate::common::QError;
use crate::common::{AtLocation, CaseInsensitiveString, ErrorEnvelope, HasLocation, Locatable};
use crate::parser::pc::common::{
    and, demand, drop_left, many_with_terminating_indicator, opt_seq2, seq3,
};
use crate::parser::pc::map::map;
use crate::parser::pc::{read, read_if, Reader, ReaderResult, Undo};
use crate::parser::pc2::binary::{BinaryParser, LeftAndOptRight, OptLeftAndRight};
use crate::parser::pc2::many::{ManyParser, OneOrMoreDelimited};
use crate::parser::pc2::text::{
    letters_or_digits_or_dots_p, letters_or_digits_p, letters_p, string_p, TextParser, Whitespace,
};
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::unary_fn::{OrThrowVal, UnaryFnParser};
use crate::parser::pc2::{if_p, item_p, Item, Parser};
use crate::parser::types::{Keyword, Name, QualifiedName};

// ========================================================
// Undo support
// ========================================================

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

pub fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

pub fn is_letter(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')
}

pub fn is_non_leading_identifier_without_dot(ch: char) -> bool {
    is_letter(ch) || is_digit(ch)
}

pub fn is_non_leading_identifier_with_dot(ch: char) -> bool {
    is_non_leading_identifier_without_dot(ch) || (ch == '.')
}

pub fn is_symbol(ch: char) -> bool {
    (ch > ' ' && ch < '0')
        || (ch > '9' && ch < 'A')
        || (ch > 'Z' && ch < 'a')
        || (ch > 'z' && ch <= '~')
}

pub fn any_letter<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, char, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
{
    read_if(is_letter)
}

/// Reads any identifier. Note that the result might be a keyword.
/// An identifier must start with a letter and consists of letters, numbers and the dot.
pub fn identifier_with_dot<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char>,
{
    letters_p()
        .and_opt(letters_or_digits_or_dots_p())
        .stringify()
}

#[deprecated]
pub fn any_identifier_without_dot<R, E>() -> Box<dyn Fn(R) -> ReaderResult<R, String, E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    E: 'static,
{
    identifier_without_dot_p().convert_to_fn()
}

/// Parses an identifier that does not include a dot.
/// This might be a keyword.
pub fn identifier_without_dot_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char>,
{
    letters_p().and_opt(letters_or_digits_p()).stringify()
}

#[deprecated]
pub fn keyword<R, E>(needle: Keyword) -> Box<dyn Fn(R) -> ReaderResult<R, (Keyword, String), E>>
where
    R: Reader<Item = char, Err = E> + 'static,
    E: 'static,
{
    keyword_p(needle).convert_to_fn()
}

/// Recognizes the given keyword.
pub fn keyword_p<R>(keyword: Keyword) -> impl Parser<R, Output = (Keyword, String)>
where
    R: Reader<Item = char>,
{
    string_p(keyword.as_str())
        .unless_followed_by(if_p(is_not_whole_keyword))
        .map(move |keyword_as_str| (keyword, keyword_as_str))
}

fn is_not_whole_keyword(ch: char) -> bool {
    is_non_leading_identifier_with_dot(ch) || ch == '$'
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

//
// Modify the result of a parser
//

//
// Take multiple items
//

#[deprecated]
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
#[deprecated]
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

/// Parses open and closing parenthesis around the given source.
/// Additional whitespace is allowed inside the parenthesis.
/// If the parser returns `None`, the left parenthesis is undone.
pub fn in_parenthesis_p<R, S>(source: S) -> impl Parser<R, Output = S::Output>
where
    R: Reader<Item = char, Err = QError>,
    S: Parser<R>,
{
    item_p('(')
        .followed_by_opt_ws()
        .stringify()
        .and(source)
        .and_demand(
            item_p(')')
                .preceded_by_opt_ws()
                .or_syntax_error("Expected: closing parenthesis"),
        )
        .keep_middle()
}

/// Offers chaining methods for parsers specific to rusty_basic.
pub trait PcSpecific<R: Reader<Item = char, Err = QError>>: Parser<R> {
    /// Throws a syntax error if this parser returns `None`.
    fn or_syntax_error(self, msg: &str) -> OrThrowVal<Self, QError> {
        OrThrowVal::new(self, QError::syntax_error(msg))
    }

    /// Parses one or more items provided by the given source, separated by commas.
    /// Trailing commas are not allowed. Space is allowed around commas.
    fn csv(
        self,
    ) -> OneOrMoreDelimited<
        Self,
        OptLeftAndRight<Whitespace<R>, LeftAndOptRight<Item<R>, Whitespace<R>>>,
        QError,
    > {
        self.one_or_more_delimited_by(
            item_p(',').surrounded_by_opt_ws(),
            QError::syntax_error("Error: trailing comma"),
        )
    }
}

impl<R: Reader<Item = char, Err = QError>, T> PcSpecific<R> for T where T: Parser<R> {}
