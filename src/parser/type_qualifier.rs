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
        if token.kind == TokenType::ExclamationMark as i32 {
            Some(TypeQualifier::BangSingle)
        } else if token.kind == TokenType::Pound as i32 {
            Some(TypeQualifier::HashDouble)
        } else if token.kind == TokenType::DollarSign as i32 {
            Some(TypeQualifier::DollarString)
        } else if token.kind == TokenType::Percent as i32 {
            Some(TypeQualifier::PercentInteger)
        } else if token.kind == TokenType::Ampersand as i32 {
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
            if token.kind == TokenType::Dot as i32 {
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
