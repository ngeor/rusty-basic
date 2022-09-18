use crate::parser::base::parsers::{Parser, TokenPredicate};
use crate::parser::base::tokenizers::Token;
use crate::parser::specific::try_from_token_type::TryFromParser;
use crate::parser::specific::TokenType;
use crate::parser::TypeQualifier;
use std::convert::TryFrom;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p() -> impl Parser<Output = TypeQualifier> {
    TryFromParser::new()
}

struct TypeQualifierPredicate;

impl TokenPredicate for TypeQualifierPredicate {
    fn test(&self, token: &Token) -> bool {
        TypeQualifier::try_from(token.kind as TokenType).is_ok()
    }
}

pub fn type_qualifier_as_token() -> impl Parser<Output = Token> {
    TypeQualifierPredicate.parser()
}
