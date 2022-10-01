//
// TokenKindParser
//

use crate::common::{QError};
use crate::parser::pc::{any_token, ErrorProvider, OptAndPC, Parser, Token, TokenPredicate};
use crate::parser::pc_specific::TokenType;
use std::convert::TryFrom;

#[deprecated]
pub struct TokenKindParser {
    token_type: TokenType,
}

impl TokenKindParser {
    pub fn new(token_type: TokenType) -> Self {
        Self { token_type }
    }
}

impl TokenPredicate for TokenKindParser {
    fn test(&self, token: &Token) -> bool {
        token.kind == self.token_type as i32
    }
}

impl ErrorProvider for TokenKindParser {
    fn provide_error_message(&self) -> String {
        match char::try_from(self.token_type) {
            Ok(ch) => format!("Expected: {}", ch),
            _ => {
                if self.token_type == TokenType::Whitespace {
                    "Expected: whitespace".to_owned()
                } else {
                    // TODO use Display instead of Debug
                    format!("Expected: token of type {:?}", self.token_type)
                }
            }
        }
    }
}

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

pub fn any_token_of(token_type: TokenType) -> impl Parser<Output = Token> {
    any_token()
        .filter(move |token| token.kind == token_type as i32)
        .map_incomplete_err(move || QError::from(token_type))
}

fn any_token_of_ws(token_type: TokenType) -> impl Parser<Output = Token> {
    OptAndPC::new(whitespace(), any_token_of(token_type))
        .and_opt(whitespace())
        .map(|((_, t), _)| t)
}
