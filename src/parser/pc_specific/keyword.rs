use crate::common::QError;
use crate::parser::pc::{Parser, Seq2, Seq3, Token};
use crate::parser::pc_specific::{any_token_of, dollar_sign, whitespace, TokenType};
use crate::parser::Keyword;

/// Matches any keyword. Ensures that it is not followed by
/// the dollar sign, in which case it is a valid identifier.
pub fn any_keyword() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Keyword)
        .and(dollar_sign().peek().negate())
        .keep_left()
}

pub fn keyword(k: Keyword) -> impl Parser<Output = Token> {
    any_keyword()
        .filter(move |token| &k == token)
        .map_incomplete_err(QError::Expected(format!("Expected: {}", k)))
}

// TODO #[deprecated]
pub fn keyword_followed_by_whitespace_p(k: Keyword) -> impl Parser {
    Seq2::new(keyword(k), whitespace().no_incomplete())
}

pub fn keyword_pair(first: Keyword, second: Keyword) -> impl Parser {
    Seq3::new(
        keyword(first),
        whitespace().no_incomplete(),
        keyword(second).no_incomplete(),
    )
}
