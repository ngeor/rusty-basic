use super::{BlockNode, Parser, ParserError, StatementNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self) -> Result<StatementNode, ParserError> {
        if let Some(s) = self.try_parse_for_loop()? {
            Ok(s)
        } else if let Some(s) = self.try_parse_if_block()? {
            Ok(s)
        } else if let Some(s) = self.try_parse_assignment()? {
            Ok(s)
        } else if let Some(s) = self.try_parse_sub_call()? {
            Ok(s)
        } else {
            Err(ParserError::NotFound(
                "Expected statement".to_string(),
                self.buf_lexer.read()?,
            ))
        }
    }

    pub fn demand_single_line_statement(&mut self) -> Result<StatementNode, ParserError> {
        if let Some(s) = self.try_parse_assignment()? {
            Ok(s)
        } else if let Some(s) = self.try_parse_sub_call()? {
            Ok(s)
        } else {
            Err(ParserError::NotFound(
                "Expected single line statement".to_string(),
                self.buf_lexer.read()?,
            ))
        }
    }

    pub fn try_parse_statement(&mut self) -> Result<Option<StatementNode>, ParserError> {
        self.demand_statement()
            .map(|x| Some(x))
            .or_else(|e| e.not_found_to_none())
    }

    pub fn parse_block(&mut self) -> Result<BlockNode, ParserError> {
        let mut statements: BlockNode = vec![];
        loop {
            self.buf_lexer.skip_whitespace_and_eol()?;
            match self.try_parse_statement()? {
                Some(s) => statements.push(s),
                _ => break,
            }
        }
        Ok(statements)
    }
}
