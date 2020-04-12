use super::{ExpressionNode, NameNode, Parser, StatementNode};
use crate::common::CaseInsensitiveString;
use crate::lexer::{LexemeNode, LexerError};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_sub_call(&mut self) -> Result<Option<StatementNode>, LexerError> {
        match self.buf_lexer.read_ref()? {
            LexemeNode::Word(_, _) => Ok(Some(self._parse_sub_call()?)),
            _ => Ok(None),
        }
    }

    fn _parse_sub_call(&mut self) -> Result<StatementNode, LexerError> {
        let (sub_name, pos) = self.buf_lexer.demand_any_word()?;
        let found_whitespace = self.buf_lexer.skip_whitespace()?;
        let args: Vec<ExpressionNode> = if found_whitespace {
            self.parse_expression_list()?
        } else {
            vec![]
        };
        self.buf_lexer.demand_eol_or_eof()?;
        Ok(StatementNode::SubCall(
            NameNode::new(CaseInsensitiveString::new(sub_name), None, pos),
            args,
        ))
    }
}
