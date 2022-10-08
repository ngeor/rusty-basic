use crate::common::QError;
use crate::parser::pc::{NonOptParser, Parser, Token, Tokenizer};
use crate::parser::pc_specific::TokenType;
use crate::parser::TypeQualifier;

/// Returns a parser that can parse a `TypeQualifier`.
pub struct TypeQualifierParser;

impl Parser for TypeQualifierParser {
    type Output = (Token, TypeQualifier);

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        if let Some(token) = tokenizer.read()? {
            if let Some(q) = Self::map(&token) {
                return TypeQualifierPostGuardParser
                    .parse(tokenizer)
                    .map(|_| (token, q));
            } else {
                tokenizer.unread(token);
            }
        }
        Err(QError::expected("Expected: one of !, #, $, %, &"))
    }
}

impl TypeQualifierParser {
    pub fn map(token: &Token) -> Option<TypeQualifier> {
        if TokenType::ExclamationMark.matches(&token) {
            Some(TypeQualifier::BangSingle)
        } else if TokenType::Pound.matches(&token) {
            Some(TypeQualifier::HashDouble)
        } else if TokenType::DollarSign.matches(&token) {
            Some(TypeQualifier::DollarString)
        } else if TokenType::Percent.matches(&token) {
            Some(TypeQualifier::PercentInteger)
        } else if TokenType::Ampersand.matches(&token) {
            Some(TypeQualifier::AmpersandLong)
        } else {
            None
        }
    }
}

pub struct TypeQualifierPostGuardParser;

impl Parser for TypeQualifierPostGuardParser {
    type Output = ();

    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        if let Some(token) = tokenizer.read()? {
            if TokenType::Dot.matches(&token) {
                Err(QError::syntax_error(
                    "Type qualifier cannot be followed by dot",
                ))
            } else if TypeQualifierParser::map(&token).is_some() {
                Err(QError::syntax_error("Duplicate type qualifier"))
            } else {
                tokenizer.unread(token);
                Ok(())
            }
        } else {
            // EOF is fine
            Ok(())
        }
    }
}

impl NonOptParser for TypeQualifierPostGuardParser {}
