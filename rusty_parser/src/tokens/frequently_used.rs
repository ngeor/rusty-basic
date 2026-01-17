use rusty_pc::{Parser, Token};

use crate::ParserError;
use crate::input::RcStringView;
use crate::pc_specific::PaddedByWs;
use crate::tokens::{TokenType, any_symbol_of, any_symbol_of_ws, any_token_of};

/// Equal sign, surrounded by optional whitespace.
///
/// `<ws>? = <ws>?`
pub fn equal_sign_ws() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of_ws!('=')
}

/// Comma, surrounded by optional whitespace.
///
/// `<ws>? , <ws>?`
pub fn comma_ws() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of_ws!(',')
}

/// Star (*), surrounded by optional whitespace.
///
/// `<ws>? * <ws>?`
pub fn star_ws() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of_ws!('*')
}

/// Colon.
///
/// `:`
pub fn colon() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of!(':')
}

/// Colon, surrounded by optional whitespace.
///
/// `<ws>? : <ws>?`
pub fn colon_ws() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of_ws!(':')
}

/// Minus sign.
///
/// `-`
pub fn minus_sign() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of!('-')
}

/// Dot.
///
/// `.`
pub fn dot() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of!('.')
}

/// Pound.
///
/// `#`
pub fn pound() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of!('#')
}

/// Dollar sign.
///
/// `$`
pub fn dollar_sign() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of!('$')
}

/// Semicolon.
///
/// `<ws>? ; <ws>?`
pub fn semicolon_ws() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_symbol_of_ws!(';')
}

pub fn digits() -> impl Parser<RcStringView, Output = Token, Error = ParserError> {
    any_token_of!(TokenType::Digits)
}
