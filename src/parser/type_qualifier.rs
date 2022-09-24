use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::TypeQualifier;
use std::convert::TryFrom;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p() -> impl Parser<Output = TypeQualifier> {
    TryFromParser::new()
}

struct TypeQualifierPredicate;

impl TokenPredicate for TypeQualifierPredicate {
    fn test(&self, token: &Token) -> bool {
        TokenType::try_from(token.kind)
            .and_then(TypeQualifier::try_from)
            .is_ok()
    }
}

pub fn type_qualifier_as_token() -> impl Parser<Output = Token> {
    TypeQualifierPredicate.parser()
}
