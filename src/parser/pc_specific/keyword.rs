use crate::common::QError;
use crate::parser::pc::{any_token, Parser, Seq2, Seq3, Token};
use crate::parser::pc_specific::{any_token_of, dollar_sign, whitespace, TokenType};
use crate::parser::Keyword;

// TODO review usages of TokenType::Keyword

/// Matches any keyword. Ensures that it is not followed by
/// the dollar sign, in which case it is a valid identifier.
pub fn any_keyword() -> impl Parser<Output = Token> {
    dollar_sign_check(any_token_of(TokenType::Keyword))
}

fn dollar_sign_check(parser: impl Parser<Output = Token>) -> impl Parser<Output = Token> {
    parser.and(dollar_sign().peek().negate()).keep_left()
}

/// Matches the specific keyword. Ensures that it is not followed
/// by the dollar sign, in which case it is a valid identifier.
pub fn keyword(k: Keyword) -> impl Parser<Output = Token> {
    dollar_sign_check(keyword_unchecked(k))
}

fn keyword_unchecked(k: Keyword) -> impl Parser<Output = Token> {
    any_token()
        .filter(move |token| &k == token)
        .map_incomplete_err(QError::Expected(format!("Expected: {}", k)))
}

// TODO 1. rename to keyword_ws like expressions 2. add ws_keyword and ws_keyword_ws
pub fn keyword_followed_by_whitespace_p(k: Keyword) -> impl Parser {
    Seq2::new(keyword_unchecked(k), whitespace().no_incomplete())
}

// TODO add keyword_pair_ws
pub fn keyword_pair(first: Keyword, second: Keyword) -> impl Parser {
    Seq3::new(
        keyword_unchecked(first),
        whitespace().no_incomplete(),
        keyword(second).no_incomplete(),
    )
}

pub fn any_keyword_with_dollar_sign() -> impl Parser<Output = (Token, Token)> {
    any_token_of(TokenType::Keyword).and(dollar_sign())
}

pub fn keyword_dollar_sign(k: Keyword) -> impl Parser<Output = (Token, Token)> {
    any_keyword_with_dollar_sign().filter(move |(token, _)| &k == token)
}
