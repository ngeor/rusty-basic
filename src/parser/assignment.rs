use super::*;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_assignment(&mut self) -> Result<Option<Statement>> {
        self.buf_lexer.mark();
        let left_side = self._try_parse_left_side_of_assignment()?;
        match left_side {
            Some(n) => {
                self.buf_lexer.clear();
                let exp = self.demand_expression()?;
                self.buf_lexer.skip_whitespace()?;
                self.buf_lexer.demand_eol_or_eof()?;
                Ok(Some(Statement::Assignment(n, exp)))
            },
            None => {
                self.buf_lexer.backtrack();
                Ok(None)
            }
        }
    }

    fn _try_parse_left_side_of_assignment(&mut self) -> Result<Option<NameWithTypeQualifier>> {
        match self.try_parse_name_with_type_qualifier()? {
            Some(n) => {
                self.buf_lexer.skip_whitespace()?;
                if self.buf_lexer.try_consume_symbol('=')? {
                    self.buf_lexer.skip_whitespace()?;
                    Ok(Some(n))
                } else {
                    Ok(None)
                }
            },
            None => {
                Ok(None)
            }
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
        let result = parser.try_parse_assignment().unwrap().unwrap();
        assert_eq!(
            result,
            Statement::Assignment(
                NameWithTypeQualifier::new_unqualified("A"),
                Expression::IntegerLiteral(42)
            )
        );
    }

    #[test]
    fn test_sub_call_is_not_mistaken_for_assignment() {
        let input = "PRINT 42";
        let mut parser = Parser::from(input);
        let result = parser.try_parse_assignment().unwrap();
        assert_eq!(result, None);
        parser.try_parse_sub_call().unwrap().unwrap();
    }
}
