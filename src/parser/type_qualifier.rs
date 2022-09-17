use crate::parser::base::parsers::{alt5, filter_token_by_kind_opt, map, Parser};
use crate::parser::specific::TokenType;
use crate::parser::TypeQualifier;

/// Returns a parser that can parse a `TypeQualifier`.
pub fn type_qualifier_p<R>() -> impl Parser<Output = TypeQualifier> {
    alt5(
        map(filter_token_by_kind_opt(TokenType::ExclamationMark), |_| TypeQualifier::BangSingle),
        map(filter_token_by_kind_opt(TokenType::Pound), |_| TypeQualifier::HashDouble),
        map(filter_token_by_kind_opt(TokenType::Percent), |_| TypeQualifier::PercentInteger),
        map(filter_token_by_kind_opt(TokenType::Ampersand), |_| TypeQualifier::AmpersandLong),
        map(filter_token_by_kind_opt(TokenType::DollarSign),|_|  TypeQualifier::DollarString)
    )
}
