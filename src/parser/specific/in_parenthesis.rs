//
// In Parenthesis
//

use crate::common::QError;
use crate::parser::base::parsers::{HasOutput, NonOptParser, Parser};
use crate::parser::base::tokenizers::{Token, Tokenizer};
use crate::parser::specific::TokenType;

pub fn in_parenthesis_opt<P>(parser: P, accept_empty_parenthesis: bool) -> InParenthesisOpt<P> {
    InParenthesisOpt {
        parser,
        accept_empty_parenthesis,
    }
}

pub fn in_parenthesis_non_opt<P>(parser: P) -> InParenthesisNonOpt<P> {
    InParenthesisNonOpt { parser }
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

pub struct InParenthesisOpt<P> {
    parser: P,
    accept_empty_parenthesis: bool,
}

impl<P> HasOutput for InParenthesisOpt<P>
where
    P: HasOutput,
{
    type Output = P::Output;
}

impl<P> Parser for InParenthesisOpt<P>
where
    P: Parser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match left_paren(tokenizer)? {
            Some(token) => {
                let opt_result = self.parser.parse(tokenizer)?;
                if opt_result.is_none() && !self.accept_empty_parenthesis {
                    tokenizer.unread(token);
                    Ok(None)
                } else {
                    right_paren(tokenizer)?;
                    Ok(opt_result)
                }
            }
            None => Ok(None),
        }
    }
}

pub struct InParenthesisNonOpt<P> {
    parser: P,
}

impl<P> HasOutput for InParenthesisNonOpt<P>
where
    P: HasOutput,
{
    type Output = P::Output;
}

impl<P> Parser for InParenthesisNonOpt<P>
where
    P: NonOptParser,
{
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match left_paren(tokenizer)? {
            Some(_) => {
                let result = self.parser.parse_non_opt(tokenizer)?;
                right_paren(tokenizer)?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }
}

impl<P> NonOptParser for InParenthesisNonOpt<P>
where
    P: NonOptParser,
{
    fn parse_non_opt(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match left_paren(tokenizer)? {
            Some(_) => {
                let result = self.parser.parse_non_opt(tokenizer)?;
                right_paren(tokenizer)?;
                Ok(result)
            }
            _ => Err(QError::syntax_error("Expected: opening parenthesis")),
        }
    }
}
