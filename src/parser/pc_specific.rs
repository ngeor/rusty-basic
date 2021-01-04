use crate::common::CaseInsensitiveString;
/// Parser combinators specific to this project (e.g. for keywords)
use crate::common::QError;
use crate::parser::pc2::binary::{BinaryParser, LeftAndOptRight, OptLeftAndRight};
use crate::parser::pc2::many::{ManyParser, OneOrMoreDelimited};
use crate::parser::pc2::text::{
    letters_or_digits_or_dots_p, letters_or_digits_p, letters_p, string_p, TextParser, Whitespace,
};
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::unary_fn::{OrThrowVal, UnaryFnParser};
use crate::parser::pc2::{if_p, item_p, Item, Parser, Reader, Undo};
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

/// Parses an identifier that does not include a dot.
/// This might be a keyword.
pub fn identifier_without_dot_p<R>() -> impl Parser<R, Output = String>
where
    R: Reader<Item = char>,
{
    letters_p().and_opt(letters_or_digits_p()).stringify()
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

// TODO: add keyword_pair_p e.g. keyword_pair_p(End, Function)
// TODO: add keywords_p e.g. keywords_p([Integer, Long, Single, Double])

//
// Take multiple items
//

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
pub trait PcSpecific<R>: Parser<R> + Sized
where
    R: Reader<Item = char, Err = QError>,
{
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
