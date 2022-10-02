use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::TypeQualifier;
use std::convert::TryFrom;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p() -> impl Parser<Output = TypeQualifier> {
    TryFromParser::new()
}

pub fn type_qualifier_as_token() -> impl Parser<Output = Token> {
    any_token()
        .filter(|token| {
            TokenType::try_from(token.kind)
                .and_then(TypeQualifier::try_from)
                .is_ok()
        })
        .map_incomplete_err(QError::expected("Expected: type qualifier"))
}
