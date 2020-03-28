use super::{Block, Parser, Expression, NameWithTypeQualifier};
use crate::common::Result;
use std::io::BufRead;
use super::if_block::IfBlock;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    SubCall(String, Vec<Expression>),
    ForLoop(
        /// The counter of the loop
        NameWithTypeQualifier,
        /// The lower bound
        Expression,
        /// The upper bound
        Expression,
        /// The statements to execute
        Block,
    ),
    IfBlock(IfBlock),
    /// Assignment to a variable e.g. ANSWER = 42
    Assignment(NameWithTypeQualifier, Expression)
}

impl Statement {
    pub fn sub_call<S: AsRef<str>>(name: S, args: Vec<Expression>) -> Statement {
        Statement::SubCall(name.as_ref().to_string(), args)
    }
}

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self) -> Result<Statement> {
        match self.try_parse_statement() {
            Ok(Some(x)) => Ok(x),
            Ok(None) => Err(format!("Expected statement, found {:?}", self.buf_lexer.read()?)),
            Err(e) => Err(e)
        }
    }

    pub fn try_parse_statement(&mut self) -> Result<Option<Statement>> {
        if let Some(s) = self.try_parse_for_loop()? {
            Ok(Some(s))
        } else if let Some(s) = self.try_parse_if_block()? {
            Ok(Some(s))
        } else if let Some(s) = self.try_parse_assignment()? {
            Ok(Some(s))
        } else if let Some(s) = self.try_parse_sub_call()? {
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }

    pub fn parse_block(&mut self) -> Result<Block> {
        let mut statements: Block = vec![];
        loop {
            self.buf_lexer.skip_whitespace_and_eol()?;
            match self.try_parse_statement()? {
                Some(s) => statements.push(s),
                None => break
            }
        }
        Ok(statements)
    }
}
