use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::TypeQualifier;
use std::convert::TryFrom;

/// Returns a parser that can parse a `TypeQualifier`.

pub fn token_to_type_qualifier(token: Token) -> Result<TypeQualifier, QError> {
    TokenType::try_from(token.kind).and_then(TypeQualifier::try_from)
}

pub fn is_type_qualifier_token(token: &Token) -> bool {
    TokenType::try_from(token.kind)
        .and_then(TypeQualifier::try_from)
        .is_ok()
}

pub fn type_qualifier_as_token() -> impl Parser<Output = Token> {
    any_token()
        .filter(is_type_qualifier_token)
        .map_incomplete_err(QError::expected("Expected: type qualifier"))
}
