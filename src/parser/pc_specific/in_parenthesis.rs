//! In Parenthesis

use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::TokenType;
use crate::parser_declaration;

/// In parser mode, returns Some if the opening parenthesis is present
/// AND the decorated parser has a value.
pub fn in_parenthesis<P>(parser: P) -> InParenthesisParser<P> {
    InParenthesisParser::new(parser)
}

parser_declaration!(struct InParenthesisParser);

impl<P> Parser for InParenthesisParser<P>
where
    P: Parser,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        if let Some(_) = left_paren(tokenizer)? {
            let value = self.parser.parse(tokenizer)?;
            right_paren(tokenizer)?;
            Ok(value)
        } else {
            Err(QError::expected("Expected: ("))
        }
    }
}

fn left_paren(tokenizer: &mut impl Tokenizer) -> Result<Option<Token>, QError> {
    match tokenizer.read()? {
        Some(token) => {
            if token.kind == TokenType::LParen as i32 {
                Ok(Some(token))
            } else {
                tokenizer.unread(token);
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn right_paren(tokenizer: &mut impl Tokenizer) -> Result<Token, QError> {
    match tokenizer.read()? {
        Some(token) => {
            if token.kind == TokenType::RParen as i32 {
                Ok(token)
            } else {
                tokenizer.unread(token);
                Err(QError::syntax_error("Expected: closing parenthesis"))
            }
        }
        None => Err(QError::InputPastEndOfFile),
    }
}
