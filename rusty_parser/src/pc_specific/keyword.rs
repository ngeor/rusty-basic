use std::collections::BTreeSet;

use rusty_pc::*;

use crate::input::RcStringView;
use crate::tokens::{TokenType, any_token, keyword_ignoring, whitespace_ignoring};
use crate::{Keyword, ParseError};

// TODO review usages of TokenType::Keyword

/// Matches the specific keyword.
///
/// Ensures that it is not followed by a dollar sign, in which case it is a valid identifier.
/// Example: `NAME$` is a valid variable name, even though `NAME` is a keyword.
pub fn keyword(
    keyword: Keyword,
) -> impl Parser<RcStringView, Output = Keyword, Error = ParseError> {
    keyword_of!(keyword)
}

// Parses one of the given keywords.

macro_rules! keyword_of {
    (
        $($keyword:expr),+
        $(,)?
    ) => {
        $crate::pc_specific::keyword_p([ $($keyword),+ ], false)
    };
}

pub(crate) use keyword_of;

/// Parses on of the given keywords, possibly treating EOF as a fatal error.
pub fn keyword_p(
    keywords: impl IntoIterator<Item = Keyword>,
    eof_is_fatal: bool,
) -> impl Parser<RcStringView, Output = Keyword, Error = ParseError> {
    KeywordParser::new(any_token(), keywords, eof_is_fatal)
}

/// Parses the given keyword, followed by mandatory whitespace.
pub fn keyword_ws_p(k: Keyword) -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    keyword_ignoring(k).and(whitespace_ignoring().no_incomplete(), IgnoringBothCombiner)
}

// TODO add keyword_pair_ws
/// Parses the first keyword, followed by mandatory whitespace,
/// followed by the second keyword. If the first keyword is parsed,
/// both the whitespace and the second keyword must be parsed.
pub fn keyword_pair(
    first: Keyword,
    second: Keyword,
) -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    keyword_ws_p(first).and(
        keyword_ignoring(second).no_incomplete(),
        IgnoringBothCombiner,
    )
}

/// Parses the specific keyword, ensuring it's not followed by a dollar sign.
/// See [keyword].
pub struct KeywordParser<P> {
    parser: P,
    // using a BTreeSet so that the generated error message is predictable (keywords sorted)
    keywords: BTreeSet<Keyword>,

    /// If true, EOF will be reported as a fatal error
    eof_is_fatal: bool,
}

impl<P> KeywordParser<P> {
    pub fn new(parser: P, keywords: impl IntoIterator<Item = Keyword>, eof_is_fatal: bool) -> Self {
        let mut keyword_set: BTreeSet<Keyword> = BTreeSet::new();
        for keyword in keywords {
            keyword_set.insert(keyword);
        }
        Self {
            parser,
            keywords: keyword_set,
            eof_is_fatal,
        }
    }
}

impl<I, C, P> Parser<I, C> for KeywordParser<P>
where
    I: Clone,
    P: Parser<I, C, Output = Token, Error = ParseError>,
{
    type Output = Keyword;
    type Error = ParseError;

    fn parse(&self, input: I) -> ParseResult<I, Keyword, ParseError> {
        let original_input = input.clone();
        match self.parser.parse(input) {
            Ok((input, token)) => match self.accept_token(&token) {
                Some(keyword) => Ok((input, keyword)),
                None => self.to_syntax_err(original_input, false),
            },
            Err((false, _, _)) => self.to_syntax_err(original_input, self.eof_is_fatal),
            Err(err) => Err(err),
        }
    }
}

impl<C, P> SetContext<C> for KeywordParser<P>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
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

    fn to_syntax_err<I, O>(&self, input: I, fatal: bool) -> ParseResult<I, O, ParseError> {
        let mut msg = String::from("Expected: ");
        let mut is_first = true;
        for k in &self.keywords {
            if is_first {
                is_first = false;
            } else {
                msg.push_str(" or ");
            }

            // doing &.to_string() to get it in uppercase
            msg.push_str(&k.to_string());
        }

        Err((fatal, input, ParseError::SyntaxError(msg)))
    }
}
