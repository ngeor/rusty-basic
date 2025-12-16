use crate::pc::{any_token, peek_token, Parser, Seq2, Seq3, Token, Tokenizer};
use crate::pc_specific::{any_token_of, dollar_sign, whitespace, TokenType};
use crate::{Keyword, ParseError};

// TODO review usages of TokenType::Keyword

fn dollar_sign_check<I: Tokenizer + 'static>(
    parser: impl Parser<I, Output = Token>,
) -> impl Parser<I, Output = Token> {
    parser
        .and(peek_token().and_then_ok_err(
            |t| {
                if TokenType::DollarSign.matches(&t) {
                    Err(ParseError::Incomplete)
                } else {
                    Ok(())
                }
            },
            |_| Ok(()),
        ))
        .keep_left()
}

/// Matches the specific keyword. Ensures that it is not followed
/// by the dollar sign, in which case it is a valid identifier.
pub fn keyword<I: Tokenizer + 'static>(k: Keyword) -> impl Parser<I, Output = Token> {
    dollar_sign_check(keyword_unchecked(k))
}

fn keyword_unchecked<I: Tokenizer + 'static>(k: Keyword) -> impl Parser<I, Output = Token> {
    any_token()
        .filter(move |token| &k == token)
        .map_incomplete_err(ParseError::Expected(format!("Expected: {}", k)))
}

// TODO 1. rename to keyword_ws like expressions 2. add ws_keyword and ws_keyword_ws
pub fn keyword_followed_by_whitespace_p<I: Tokenizer + 'static>(k: Keyword) -> impl Parser<I> {
    Seq2::new(keyword_unchecked(k), whitespace().no_incomplete())
}

// TODO add keyword_pair_ws
pub fn keyword_pair<I: Tokenizer + 'static>(first: Keyword, second: Keyword) -> impl Parser<I> {
    Seq3::new(
        keyword_unchecked(first),
        whitespace().no_incomplete(),
        keyword(second).no_incomplete(),
    )
}

pub fn any_keyword_with_dollar_sign<I: Tokenizer + 'static>(
) -> impl Parser<I, Output = (Token, Token)> {
    any_token_of(TokenType::Keyword).and(dollar_sign())
}

pub fn keyword_dollar_sign<I: Tokenizer + 'static>(
    k: Keyword,
) -> impl Parser<I, Output = (Token, Token)> {
    any_keyword_with_dollar_sign().filter(move |(token, _)| &k == token)
}
