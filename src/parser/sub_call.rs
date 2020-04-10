use super::*;

fn _is_allowed_sub_name(word: &String) -> bool {
    word != "NEXT"
        && word != "END"
        && word != "DECLARE"
        && word != "IF"
        && word != "ELSEIF"
        && word != "ELSE"
}

impl<T: BufRead> Parser<T> {
    pub fn try_parse_sub_call(&mut self) -> Result<Option<StatementNode>, LexerError> {
        match self.buf_lexer.read()? {
            LexemeNode::Word(w, _) => {
                if _is_allowed_sub_name(&w) {
                    Ok(Some(self._parse_sub_call()?))
                } else {
                    Ok(None)
                }
            }
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
            NameNode::new(sub_name, None, pos),
            args,
        ))
    }
}
