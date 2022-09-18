use crate::parser::base::parsers::Parser;
use crate::parser::specific::{map_tokens, TokenType};
use crate::parser::TypeQualifier;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p() -> impl Parser<Output = TypeQualifier> {
    map_tokens(&[
        (TokenType::ExclamationMark, TypeQualifier::BangSingle),
        (TokenType::Pound, TypeQualifier::HashDouble),
        (TokenType::Percent, TypeQualifier::PercentInteger),
        (TokenType::Ampersand, TypeQualifier::AmpersandLong),
        (TokenType::DollarSign, TypeQualifier::DollarString),
    ])
}
