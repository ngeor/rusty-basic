/// Parser combinators specific to this project (e.g. for keywords)
use crate::common::*;
use crate::parser::pc::*;
use crate::parser::types::*;

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

// Reads any identifier. Note that the result might be a keyword.
// An identifier must start with a letter and consists of letters, numbers and the dot.
crate::char_sequence_p!(
    IdentifierWithDot,
    identifier_with_dot,
    is_letter,
    is_letter_or_digit_or_dot
);

fn is_letter_or_digit_or_dot(ch: char) -> bool {
    is_letter(ch) || is_digit(ch) || ch == '.'
}

// Parses an identifier that does not include a dot.
// This might be a keyword.
crate::char_sequence_p!(
    IdentifierWithoutDot,
    identifier_without_dot_p,
    is_letter,
    is_letter_or_digit
);

fn is_letter_or_digit(ch: char) -> bool {
    is_letter(ch) || is_digit(ch)
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
    is_letter(ch) || is_digit(ch) || ch == '.' || ch == '$'
}

// TODO: add keywords_p e.g. keywords_p([Integer, Long, Single, Double])

pub fn keyword_pair_p<R>(
    first: Keyword,
    second: Keyword,
) -> impl Parser<R, Output = (Keyword, String, Keyword, String)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(first)
        .and_demand(whitespace_p().or_syntax_error(format!("Expected: whitespace after {}", first)))
        .and_demand(keyword_p(second).or_syntax_error(format!("Expected: {}", second)))
        .map(|(((first_k, first_s), _), (second_k, second_s))| {
            (first_k, first_s, second_k, second_s)
        })
}

pub fn demand_keyword_pair_p<R>(
    first: Keyword,
    second: Keyword,
) -> impl Parser<R, Output = (Keyword, String, Keyword, String)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_pair_p(first, second).or_syntax_error(format!("Expected: {} {}", first, second))
}

//
// Take multiple items
//

/// Parses open and closing parenthesis around the given source.
/// Additional whitespace is allowed inside the parenthesis.
/// If the parser returns `None`, the left parenthesis is undone.
pub fn in_parenthesis_p<R, S>(source: S) -> impl Parser<R, Output = S::Output>
where
    R: Reader<Item = char, Err = QError> + 'static,
    S: Parser<R> + 'static,
{
    lparen_opt_ws_p()
        .and(source)
        .and_demand(
            item_p(')')
                .preceded_by_opt_ws()
                .or_syntax_error("Expected: closing parenthesis"),
        )
        .keep_middle()
}

crate::char_sequence_p!(
    LParenOptWhitespace,
    lparen_opt_ws_p,
    is_left_parenthesis,
    is_whitespace
);
fn is_left_parenthesis(ch: char) -> bool {
    ch == '('
}

/// Offers chaining methods for parsers specific to rusty_basic.
pub trait PcSpecific<R>: Parser<R> + Sized
where
    R: Reader<Item = char, Err = QError>,
{
    /// Throws a syntax error if this parser returns `None`.
    fn or_syntax_error<S: AsRef<str>>(self, msg: S) -> OrThrowVal<Self, QError> {
        OrThrowVal::new(self, QError::syntax_error(msg))
    }

    /// Parses one or more items provided by the given source, separated by commas.
    /// Trailing commas are not allowed. Space is allowed around commas.
    fn csv(self) -> OneOrMoreDelimited<Self, SurroundedByOptWhitespace<Item<R>>, QError> {
        self.one_or_more_delimited_by(
            item_p(',').surrounded_by_opt_ws(),
            QError::syntax_error("Error: trailing comma"),
        )
    }
}

impl<R: Reader<Item = char, Err = QError>, T> PcSpecific<R> for T where T: Parser<R> {}
