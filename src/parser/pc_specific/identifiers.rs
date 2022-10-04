//
// identifier or keyword
//

// TODO review name-like parsers

use crate::common::QError;
use crate::parser::pc::{any_token, Parser, Token};
use crate::parser::pc_specific::{any_token_of, TokenType};

const MAX_LENGTH: usize = 40;

pub fn identifier() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Identifier).and_then(ensure_token_list_length)
}

pub fn identifier_or_keyword() -> impl Parser<Output = Token> {
    any_token()
        .filter(|token| {
            token.kind == TokenType::Keyword as i32 || token.kind == TokenType::Identifier as i32
        })
        .and_then(ensure_token_list_length)
        .map_incomplete_err(QError::Expected(
            "Expected: identifier or keyword".to_owned(),
        ))
}

pub fn identifier_or_keyword_without_dot() -> impl Parser<Output = Token> {
    any_token()
        .filter(|token| {
            (token.kind == TokenType::Keyword as i32 || token.kind == TokenType::Identifier as i32)
                && !token.text.contains('.')
        })
        .and_then(ensure_token_list_length)
        .map_incomplete_err(QError::Expected(
            "Expected: identifier or keyword".to_owned(),
        ))
}

pub fn identifier_without_dot() -> impl Parser<Output = Token> {
    any_token()
        .filter(|token| token.kind == TokenType::Identifier as i32 && !token.text.contains('.'))
        .and_then(ensure_token_list_length)
}

fn ensure_token_list_length(token: Token) -> Result<Token, QError> {
    if token.kind == TokenType::Identifier as i32 && token.text.chars().count() > MAX_LENGTH {
        Err(QError::IdentifierTooLong)
    } else {
        Ok(token)
    }
}
