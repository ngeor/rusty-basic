use super::parse_result::ParseResult;
use super::*;

impl From<StatementNode> for ParseResult<StatementNode> {
    fn from(expr: StatementNode) -> ParseResult<StatementNode> {
        ParseResult::Match(expr)
    }
}

impl From<StatementNode> for Result<ParseResult<StatementNode>, LexerError> {
    fn from(expr: StatementNode) -> Result<ParseResult<StatementNode>, LexerError> {
        Ok(expr.into())
    }
}

impl<T: BufRead> Parser<T> {
    pub fn demand_statement(&mut self) -> Result<StatementNode, LexerError> {
        match self.try_parse_statement() {
            Ok(x) => x.demand("Expected statement"),
            Err(err) => Err(err),
        }
    }

    pub fn try_parse_statement(&mut self) -> Result<ParseResult<StatementNode>, LexerError> {
        if let Some(s) = self.try_parse_for_loop()? {
            s.into()
        } else if let Some(s) = self.try_parse_if_block()? {
            s.into()
        } else if let Some(s) = self.try_parse_assignment()? {
            s.into()
        } else if let Some(s) = self.try_parse_sub_call()? {
            s.into()
        } else if let Some(s) = self._try_parse_whitespace()? {
            s.into()
        } else {
            self.buf_lexer.read()?.into()
        }
    }

    pub fn parse_block(&mut self) -> Result<BlockNode, LexerError> {
        let mut statements: BlockNode = vec![];
        loop {
            self.buf_lexer.skip_whitespace_and_eol()?;
            match self.try_parse_statement()? {
                ParseResult::Match(s) => statements.push(s),
                _ => break,
            }
        }
        Ok(statements)
    }

    fn _try_parse_whitespace(&mut self) -> Result<Option<StatementNode>, LexerError> {
        let mut buf = String::new();
        loop {
            let lexeme = self.buf_lexer.read()?;
            match lexeme {
                LexemeNode::Whitespace(w, _) | LexemeNode::EOL(w, _) => {
                    self.buf_lexer.consume();
                    buf.push_str(&w);
                }
                _ => break,
            }
        }

        if buf.is_empty() {
            Ok(None)
        } else {
            Ok(Some(StatementNode::Whitespace(buf)))
        }
    }
}
