use std::collections::BTreeSet;

use rusty_pc::*;

use crate::input::RcStringView;
use crate::tokens::{TokenMatcher, TokenType, any_token_of, dollar_sign, whitespace};
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

macro_rules! keyword_of {
    (
        $($keyword:expr),+
        $(,)?
    ) => {
        $crate::pc_specific::KeywordParser::new($crate::tokens::any_token(), [ $($keyword),+ ])
    };
}

pub(crate) use keyword_of;

// TODO 1. rename to keyword_ws like expressions 2. add ws_keyword and ws_keyword_ws
pub fn keyword_followed_by_whitespace_p(
    k: Keyword,
) -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    seq2(keyword(k), whitespace(), |_, _| ())
}

// TODO add keyword_pair_ws
pub fn keyword_pair(
    first: Keyword,
    second: Keyword,
) -> impl Parser<RcStringView, Error = ParseError> {
    seq3(keyword(first), whitespace(), keyword(second), |_, _, _| ())
}

pub fn any_keyword_with_dollar_sign()
-> impl Parser<RcStringView, Output = (Token, Token), Error = ParseError> {
    any_token_of!(TokenType::Keyword).and_tuple(dollar_sign())
}

pub fn keyword_dollar_sign(
    k: Keyword,
) -> impl Parser<RcStringView, Output = (Token, Token), Error = ParseError> {
    any_keyword_with_dollar_sign().filter_or_err(
        move |(token, _)| k.matches_token(token),
        ParseError::SyntaxError(format!("Expected: {}", k)),
    )
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
    I: Clone,
    P: Parser<I, C, Output = Token, Error = ParseError>,
{
    type Output = Keyword;
    type Error = ParseError;

    fn parse(&self, input: I) -> ParseResult<I, Keyword, ParseError> {
        let original_input = input.clone();
        match self.parser.parse(input) {
            Ok((input, token)) => {
                match self.accept_token(&token) {
                    Some(keyword) => {
                        // found the keyword, but make sure it's not followed by a dollar sign
                        self.ensure_no_trailing_dollar_sign(input.clone(), original_input)?;
                        Ok((input, keyword))
                    }
                    None => self.to_syntax_err(original_input),
                }
            }
            Err((false, _, _)) => self.to_syntax_err(original_input),
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

    fn to_syntax_err<I, O>(&self, input: I) -> ParseResult<I, O, ParseError> {
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

        Err((false, input, ParseError::SyntaxError(msg)))
    }

    // TODO move this check into `any_token`
    fn ensure_no_trailing_dollar_sign<I, C>(
        &self,
        input: I,
        original_input: I,
    ) -> ParseResult<I, (), ParseError>
    where
        I: Clone,
        P: Parser<I, C, Output = Token, Error = ParseError>,
    {
        let input_after_keyword = input.clone();

        match self.parser.parse(input) {
            Ok((_, token_after_keyword)) => {
                if '$'.matches_token(&token_after_keyword) {
                    // undo everything, let another parser pick up `NAME$`, which is a valid variable name, despite `NAME` being also a keyword
                    self.to_syntax_err(original_input)
                } else {
                    Ok((input_after_keyword, ()))
                }
            }
            Err((false, _, _)) => Ok((input_after_keyword, ())),
            Err(err) => Err(err),
        }
    }
}
