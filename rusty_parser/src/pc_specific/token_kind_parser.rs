//
// TokenKindParser
//

use crate::pc::{any_token, Parser, Token, Tokenizer};
use crate::pc_specific::TokenType;

/// Equal sign, surrounded by optional whitespace.
///
/// `<ws>? = <ws>?`
pub fn equal_sign<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of_ws(TokenType::Equals)
}

/// Comma, surrounded by optional whitespace.
///
/// `<ws>? , <ws>?`
pub fn comma<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of_ws(TokenType::Comma)
}

/// Star (*), surrounded by optional whitespace.
///
/// `<ws>? * <ws>?`
pub fn star<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of_ws(TokenType::Star)
}

/// Colon.
///
/// `:`
pub fn colon<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of(TokenType::Colon)
}

/// Colon, surrounded by optional whitespace.
///
/// `<ws>? : <ws>?`
pub fn colon_ws<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of_ws(TokenType::Colon)
}

/// Minus sign.
///
/// `-`
pub fn minus_sign<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of(TokenType::Minus)
}

/// Dot.
///
/// `.`
pub fn dot<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of(TokenType::Dot)
}

/// Pound.
///
/// `#`
pub fn pound<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of(TokenType::Pound)
}

/// Dollar sign.
///
/// `$`
pub fn dollar_sign<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of(TokenType::DollarSign)
}

/// Semicolon.
///
/// `<ws>? ; <ws>?`
pub fn semicolon<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of_ws(TokenType::Semicolon)
}

/// Whitespace.
pub fn whitespace<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of(TokenType::Whitespace)
}

/// Optional whitespace.
fn opt_whitespace<I: Tokenizer + 'static>() -> impl Parser<I, Output = Option<Token>> {
    whitespace().allow_none()
}

pub fn digits<I: Tokenizer + 'static>() -> impl Parser<I, Output = Token> {
    any_token_of(TokenType::Digits)
}

pub fn any_token_of<I: Tokenizer + 'static>(
    token_type: TokenType,
) -> impl Parser<I, Output = Token> {
    any_token()
        .filter(move |token| token_type.matches(token))
        .with_expected_message(move || token_type.to_error_message())
}

fn any_token_of_ws<I: Tokenizer + 'static>(
    token_type: TokenType,
) -> impl Parser<I, Output = Token> {
    any_token_of(token_type).surround(opt_whitespace(), opt_whitespace())
}

pub fn any_token_of_two<I: Tokenizer + 'static>(
    token_type1: TokenType,
    token_type2: TokenType,
) -> impl Parser<I, Output = Token> {
    any_token().filter(move |token| token_type1.matches(token) || token_type2.matches(token))
}
