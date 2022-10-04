//
// TokenKindParser
//

use crate::common::QError;
use crate::parser::pc::{any_token, OptAndPC, Parser, Token};
use crate::parser::pc_specific::TokenType;

/// Equal sign, surrounded by optional whitespace.
///
/// `<ws>? = <ws>?`
pub fn equal_sign() -> impl Parser<Output = Token> {
    any_token_of_ws(TokenType::Equals)
}

/// Comma, surrounded by optional whitespace.
///
/// `<ws>? , <ws>?`
pub fn comma() -> impl Parser<Output = Token> {
    any_token_of_ws(TokenType::Comma)
}

/// Star (*), surrounded by optional whitespace.
///
/// `<ws>? * <ws>?`
pub fn star() -> impl Parser<Output = Token> {
    any_token_of_ws(TokenType::Star)
}

/// Colon.
///
/// `:`
pub fn colon() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Colon)
}

/// Colon, surrounded by optional whitespace.
///
/// `<ws>? : <ws>?`
pub fn colon_ws() -> impl Parser<Output = Token> {
    any_token_of_ws(TokenType::Colon)
}

/// Dollar sign.
///
/// `$`
pub fn dollar_sign() -> impl Parser<Output = Token> {
    any_token_of(TokenType::DollarSign)
}

/// Minus sign.
///
/// `-`
pub fn minus_sign() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Minus)
}

/// Dot.
///
/// `.`
pub fn dot() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Dot)
}

/// Pound.
///
/// `#`
pub fn pound() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Pound)
}

/// Semicolon.
///
/// `;`
pub fn semicolon() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Semicolon)
}

/// Whitespace.
pub fn whitespace() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Whitespace)
}

pub fn digits() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Digits)
}

pub fn any_token_of(token_type: TokenType) -> impl Parser<Output = Token> {
    any_token()
        .filter(move |token| token.kind == token_type as i32)
        .map_incomplete_err(QError::from(token_type))
}

fn any_token_of_ws(token_type: TokenType) -> impl Parser<Output = Token> {
    OptAndPC::new(whitespace(), any_token_of(token_type))
        .and_opt(whitespace())
        .map(|((_, t), _)| t)
}
