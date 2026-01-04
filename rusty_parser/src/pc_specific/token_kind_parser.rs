//
// TokenKindParser
//

use rusty_pc::{And, Parser, ToOption, Token};

use crate::ParseError;
use crate::input::RcStringView;
use crate::pc_specific::{TokenType, any_token_of};

/// Equal sign, surrounded by optional whitespace.
///
/// `<ws>? = <ws>?`
pub fn equal_sign() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of_ws(TokenType::Equals)
}

/// Comma, surrounded by optional whitespace.
///
/// `<ws>? , <ws>?`
pub fn comma() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of_ws(TokenType::Comma)
}

/// Star (*), surrounded by optional whitespace.
///
/// `<ws>? * <ws>?`
pub fn star() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of_ws(TokenType::Star)
}

/// Colon.
///
/// `:`
pub fn colon() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of!(TokenType::Colon)
}

/// Colon, surrounded by optional whitespace.
///
/// `<ws>? : <ws>?`
pub fn colon_ws() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of_ws(TokenType::Colon)
}

/// Minus sign.
///
/// `-`
pub fn minus_sign() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of!(TokenType::Minus)
}

/// Dot.
///
/// `.`
pub fn dot() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of!(TokenType::Dot)
}

/// Pound.
///
/// `#`
pub fn pound() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of!(TokenType::Pound)
}

/// Dollar sign.
///
/// `$`
pub fn dollar_sign() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of!(TokenType::DollarSign)
}

/// Semicolon.
///
/// `<ws>? ; <ws>?`
pub fn semicolon() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of_ws(TokenType::Semicolon)
}

/// Whitespace.
pub fn whitespace() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of!(TokenType::Whitespace)
}

/// Optional whitespace.
pub fn opt_whitespace() -> impl Parser<RcStringView, Output = Option<Token>, Error = ParseError> {
    whitespace().to_option()
}

pub fn digits() -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of!(TokenType::Digits)
}

fn any_token_of_ws(
    token_type: TokenType,
) -> impl Parser<RcStringView, Output = Token, Error = ParseError> {
    any_token_of!(token_type).surround(opt_whitespace(), opt_whitespace())
}
