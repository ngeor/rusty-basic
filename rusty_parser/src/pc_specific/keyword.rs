use std::collections::BTreeSet;

use rusty_pc::and::IgnoringBothCombiner;
use rusty_pc::*;

use crate::input::StringView;
use crate::pc_specific::{WithExpected, whitespace_ignoring};
use crate::tokens::{TokenMatcher, TokenType, any_token};
use crate::{Keyword, ParserError};

// TODO review usages of TokenType::Keyword

/// Matches the specific keyword.
///
/// Ensures that it is not followed by a dollar sign, in which case it is a valid identifier.
/// Example: `NAME$` is a valid variable name, even though `NAME` is a keyword.
pub fn keyword(keyword: Keyword) -> impl Parser<StringView, Output = Keyword, Error = ParserError> {
    keyword_of!(keyword)
}

// Parses one of the given keywords.

macro_rules! keyword_of {
    (
        $($keyword:expr),+
        $(,)?
    ) => {
        $crate::pc_specific::keyword_p([ $($keyword),+ ])
    };
}

pub(crate) use keyword_of;

/// Parses on of the given keywords, possibly treating EOF as a fatal error.
pub fn keyword_p(
    keywords: impl IntoIterator<Item = Keyword>,
) -> impl Parser<StringView, Output = Keyword, Error = ParserError> {
    KeywordParser::new(any_token(), keywords)
}

pub fn keyword_ignoring(k: Keyword) -> impl Parser<StringView, Output = (), Error = ParserError> {
    any_token()
        .filter(move |t| k.matches_token(t))
        .map_to_unit()
        .with_expected_message(format!("Expected: {}", k))
}

/// Parses the given keyword, followed by mandatory whitespace.
pub fn keyword_ws_p(k: Keyword) -> impl Parser<StringView, Output = (), Error = ParserError> {
    keyword_ignoring(k).and(whitespace_ignoring().to_fatal(), IgnoringBothCombiner)
}

// TODO add keyword_pair_ws
/// Parses the first keyword, followed by mandatory whitespace,
/// followed by the second keyword. If the first keyword is parsed,
/// both the whitespace and the second keyword must be parsed.
pub fn keyword_pair(
    first: Keyword,
    second: Keyword,
) -> impl Parser<StringView, Output = (), Error = ParserError> {
    keyword_ws_p(first).and(keyword_ignoring(second).to_fatal(), IgnoringBothCombiner)
}

/// Parses the specific keyword, ensuring it's not followed by a dollar sign.
/// See [keyword].
pub struct KeywordParser<P> {
    parser: P,
    // using a BTreeSet so that the generated error message is predictable (keywords sorted)
    keywords: BTreeSet<Keyword>,
}

impl<P> KeywordParser<P> {
    pub fn new(parser: P, keywords: impl IntoIterator<Item = Keyword>) -> Self {
        let mut keyword_set: BTreeSet<Keyword> = BTreeSet::new();
        for keyword in keywords {
            keyword_set.insert(keyword);
        }
        Self {
            parser,
            keywords: keyword_set,
        }
    }
}

impl<I, C, P> Parser<I, C> for KeywordParser<P>
where
    I: InputTrait,
    P: Parser<I, C, Output = Token, Error = ParserError>,
{
    type Output = Keyword;
    type Error = ParserError;

    fn parse(&mut self, input: &mut I) -> Result<Keyword, ParserError> {
        let original_input = input.get_position();
        match self.parser.parse(input) {
            Ok(token) => match self.accept_token(&token) {
                Some(keyword) => Ok(keyword),
                None => {
                    input.set_position(original_input);
                    self.to_syntax_err()
                }
            },
            Err(err) if err.is_soft() => {
                input.set_position(original_input);
                self.to_syntax_err()
            }
            Err(err) => Err(err),
        }
    }

    fn set_context(&mut self, ctx: &C) {
        self.parser.set_context(ctx);
    }
}

impl<P> KeywordParser<P> {
    fn accept_token(&self, token: &Token) -> Option<Keyword> {
        if token.kind() == TokenType::Keyword.get_index() {
            // try parse the token text (this should always succeed)
            Keyword::try_from(token.as_str())
                // ignore failures (alternatively we could panic here, indicating something of TokenType::Keyword couldn't be parsed)
                .ok()
                // is it one of the desired keyword we're looking for?
                .filter(|k| self.keywords.contains(k))
        } else {
            None
        }
    }

    fn to_syntax_err<O>(&self) -> Result<O, ParserError> {
        let s = to_syntax_err(self.keywords.iter());
        Err(ParserError::expected(&s))
    }
}

/// Concatenates the given keywords into a syntax error message.
pub fn to_syntax_err<'a>(keywords: impl Iterator<Item = &'a Keyword>) -> String {
    keywords
        .map(|k| k.to_string())
        .reduce(|a, b| a + " or " + &b)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_syntax_err_zero_items() {
        let keywords: Vec<Keyword> = vec![];
        assert_eq!(to_syntax_err(keywords.iter()), "");
    }

    #[test]
    fn test_to_syntax_err_one_item() {
        let keywords = vec![Keyword::If];
        assert_eq!(to_syntax_err(keywords.iter()), "IF");
    }

    #[test]
    fn test_to_syntax_err_two_items() {
        let keywords = vec![Keyword::If, Keyword::Then];
        assert_eq!(to_syntax_err(keywords.iter()), "IF or THEN");
    }

    #[test]
    fn test_to_syntax_err_three_items() {
        let keywords = vec![Keyword::If, Keyword::Then, Keyword::Else];
        assert_eq!(to_syntax_err(keywords.iter()), "IF or THEN or ELSE");
    }
}
