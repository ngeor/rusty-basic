use super::{Block, Expression, ForLoop, IfBlock, Parser, QName};
use crate::common::Result;
use crate::lexer::Lexeme;
use std::io::BufRead;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    SubCall(String, Vec<Expression>),
    ForLoop(ForLoop),
    IfBlock(IfBlock),
    /// Assignment to a variable e.g. ANSWER = 42
    Assignment(QName, Expression),
    Whitespace(String),
}

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self) -> Result<Statement> {
        match self.try_parse_statement() {
            Ok(Some(x)) => Ok(x),
            Ok(None) => Err(format!(
                "Expected statement, found {:?}",
                self.buf_lexer.read()?
            )),
            Err(e) => Err(e),
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
        } else if let Some(s) = self._try_parse_whitespace()? {
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
                None => break,
            }
        }
        Ok(statements)
    }

    fn _try_parse_whitespace(&mut self) -> Result<Option<Statement>> {
        let mut buf = String::new();
        loop {
            let lexeme = self.buf_lexer.read()?;
            match lexeme {
                Lexeme::Whitespace(w) => {
                    self.buf_lexer.consume();
                    buf.push_str(&w);
                }
                Lexeme::EOL(w) => {
                    self.buf_lexer.consume();
                    buf.push_str(&w);
                }
                _ => break,
            }
        }

        if buf.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Statement::Whitespace(buf)))
        }
    }
}
