//! In Parenthesis

use crate::common::QError;
use crate::parser::pc::*;
use crate::parser::pc_specific::TokenType;
use crate::parser_decorator;

/// In parser mode, returns Some if the opening parenthesis is present
/// AND the decorated parser has a value.
pub fn in_parenthesis<P>(parser: P) -> InParenthesisParser<P> {
    InParenthesisParser::new(parser)
}

parser_decorator!(struct InParenthesisParser);

impl<P> Parser for InParenthesisParser<P>
where
    P: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        if let Some(_) = left_paren(tokenizer)? {
            let value = self.0.parse_non_opt(tokenizer)?;
            right_paren(tokenizer)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

impl<P> NonOptParser for InParenthesisParser<P>
where
    P: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        if let Some(_) = left_paren(tokenizer)? {
            let value = self.0.parse_non_opt(tokenizer)?;
            right_paren(tokenizer)?;
            Ok(value)
        } else {
            Err(QError::syntax_error("Expected: ("))
        }
    }
}

/// Allows missing left parenthesis and allows zero elements inside the parenthesis
pub fn in_parenthesis_allow_no_elements<P>(parser: P) -> InParenthesisAllowNoElements<P> {
    InParenthesisAllowNoElements::new(parser)
}

parser_decorator!(struct InParenthesisAllowNoElements);

impl<P> Parser for InParenthesisAllowNoElements<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        if let Some(_) = left_paren(tokenizer)? {
            let opt_value = self.0.parse(tokenizer)?;
            right_paren(tokenizer)?;
            Ok(opt_value)
        } else {
            Ok(None)
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
