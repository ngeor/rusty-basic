use crate::pc::*;
use crate::specific::pc_specific::{any_token_of, dollar_sign, whitespace, TokenType};
use crate::specific::Keyword;

// TODO review usages of TokenType::Keyword

fn dollar_sign_check(
    parser: impl Parser<RcStringView, Output = Token>,
) -> impl Parser<RcStringView, Output = Token> {
    parser.and_keep_left(peek_token().flat_map_negate_none(|input, t| {
        if TokenType::DollarSign.matches(&t) {
            default_parse_error(input)
        } else {
            Ok((input, ()))
        }
    }))
}

/// Matches the specific keyword. Ensures that it is not followed
/// by the dollar sign, in which case it is a valid identifier.
pub fn keyword(k: Keyword) -> impl Parser<RcStringView, Output = Token> {
    dollar_sign_check(keyword_unchecked(k))
}

fn keyword_unchecked(k: Keyword) -> impl Parser<RcStringView, Output = Token> {
    any_token()
        .filter(move |token| &k == token)
        .with_expected_message(format!("Expected: {}", k))
}

// TODO 1. rename to keyword_ws like expressions 2. add ws_keyword and ws_keyword_ws
pub fn keyword_followed_by_whitespace_p(k: Keyword) -> impl Parser<RcStringView> {
    Seq2::new(keyword_unchecked(k), whitespace())
}

// TODO add keyword_pair_ws
pub fn keyword_pair(first: Keyword, second: Keyword) -> impl Parser<RcStringView> {
    Seq3::new(keyword_unchecked(first), whitespace(), keyword(second))
}

pub fn any_keyword_with_dollar_sign() -> impl Parser<RcStringView, Output = (Token, Token)> {
    any_token_of(TokenType::Keyword).and_tuple(dollar_sign())
}

pub fn keyword_dollar_sign(k: Keyword) -> impl Parser<RcStringView, Output = (Token, Token)> {
    any_keyword_with_dollar_sign().filter(move |(token, _)| &k == token)
}
