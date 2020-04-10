use super::*;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_assignment(&mut self) -> Result<Option<StatementNode>, LexerError> {
        self.buf_lexer.mark();
        let left_side = self._try_parse_left_side_of_assignment()?;
        match left_side {
            Some(n) => {
                self.buf_lexer.clear();
                let exp = self.demand_expression()?;
                self.buf_lexer.skip_whitespace()?;
                self.buf_lexer.demand_eol_or_eof()?;
                Ok(Some(StatementNode::Assignment(n, exp)))
            }
            None => {
                self.buf_lexer.backtrack();
                Ok(None)
            }
        }
    }

    fn _try_parse_left_side_of_assignment(&mut self) -> Result<Option<NameNode>, LexerError> {
        match self.try_parse_name_with_type_qualifier()? {
            Some(n) => {
                self.buf_lexer.skip_whitespace()?;
                if self.buf_lexer.try_consume_symbol('=')?.is_some() {
                    self.buf_lexer.skip_whitespace()?;
                    Ok(Some(n))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_assignment() {
        let input = "A = 42";
        let mut parser = Parser::from(input);
        let result = parser
            .try_parse_assignment()
            .unwrap()
            .unwrap()
            .strip_location();
        assert_eq!(
            result,
            Statement::Assignment(Name::from("A"), Expression::IntegerLiteral(42),),
        );
    }

    #[test]
    fn test_sub_call_is_not_mistaken_for_assignment() {
        let input = "PRINT 42";
        let mut parser = Parser::from(input);
        let result = parser.try_parse_assignment().unwrap();
        match result {
            None => (),
            Some(_) => panic!("should not have mistaken sub call for assignment"),
        };
        parser.try_parse_sub_call().unwrap().unwrap();
    }
}
