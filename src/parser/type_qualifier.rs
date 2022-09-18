use crate::parser::base::parsers::{OrTrait, Parser};
use crate::parser::base::tokenizers::Token;
use crate::parser::specific::{map_tokens, TokenKindParser, TokenType};
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

pub fn type_qualifier_as_token() -> impl Parser<Output = Token> {
    TokenKindParser::new(TokenType::ExclamationMark)
        .or(TokenKindParser::new(TokenType::Pound))
        .or(TokenKindParser::new(TokenType::Percent))
        .or(TokenKindParser::new(TokenType::Ampersand))
        .or(TokenKindParser::new(TokenType::DollarSign))
}
