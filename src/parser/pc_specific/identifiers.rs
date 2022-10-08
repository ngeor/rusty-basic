//
// identifier or keyword
//

// TODO review name-like parsers

use crate::common::QError;
use crate::parser::pc::{Parser, Token};
use crate::parser::pc_specific::{any_token_of, TokenType};

const MAX_LENGTH: usize = 40;

pub fn identifier_with_dots() -> impl Parser<Output = Token> {
    any_token_of(TokenType::Identifier).and_then(ensure_token_list_length)
}

fn ensure_token_list_length(token: Token) -> Result<Token, QError> {
    if token.kind == TokenType::Identifier as i32 && token.text.chars().count() > MAX_LENGTH {
        Err(QError::IdentifierTooLong)
    } else {
        Ok(token)
    }
}
