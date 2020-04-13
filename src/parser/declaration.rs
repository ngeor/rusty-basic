use super::{FunctionDeclarationNode, NameNode, Parser, TopLevelTokenNode};
use crate::common::Location;
use crate::lexer::{Keyword, LexemeNode, LexerError};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_declaration(&mut self) -> Result<Option<TopLevelTokenNode>, LexerError> {
        match self.buf_lexer.try_consume_keyword(Keyword::Declare)? {
            Some(pos) => self._parse_declaration(pos).map(|x| Some(x)),
            None => Ok(None),
        }
    }

    fn _parse_declaration(
        &mut self,
        declare_pos: Location,
    ) -> Result<TopLevelTokenNode, LexerError> {
        self.buf_lexer.demand_whitespace()?;
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Keyword(Keyword::Function, _, _) => {
                self.buf_lexer.consume();
                self.buf_lexer.demand_whitespace()?;
                let function_name = self.demand_name_with_type_qualifier()?;
                self.buf_lexer.skip_whitespace()?;
                let function_arguments: Vec<NameNode> = self.parse_declaration_parameters()?;
                self.buf_lexer.demand_eol_or_eof()?;
                Ok(TopLevelTokenNode::FunctionDeclaration(
                    FunctionDeclarationNode::new(function_name, function_arguments, declare_pos),
                ))
            }
            _ => Err(LexerError::Unexpected(
                "Unknown declaration".to_string(),
                next,
            )),
        }
    }

    pub fn parse_declaration_parameters(&mut self) -> Result<Vec<NameNode>, LexerError> {
        let mut function_arguments: Vec<NameNode> = vec![];
        if self.buf_lexer.try_consume_symbol('(')?.is_some() {
            self.buf_lexer.skip_whitespace()?;
            let mut is_first_parameter = true;
            while self.buf_lexer.try_consume_symbol(')')?.is_none() {
                if is_first_parameter {
                    is_first_parameter = false;
                } else {
                    self.buf_lexer.demand_symbol(',')?;
                    self.buf_lexer.skip_whitespace()?;
                }
                function_arguments.push(self.demand_name_with_type_qualifier()?);
                self.buf_lexer.skip_whitespace()?;
            }
        }
        Ok(function_arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;

    macro_rules! assert_function_declaration {
        ($input:expr, $expected_function_name:expr, $expected_params:expr) => {
            match parse_single_top_level_token_node($input) {
                TopLevelTokenNode::FunctionDeclaration(f) => {
                    assert_eq!(&f.name, $expected_function_name, "Function name mismatch");
                    let x = $expected_params;
                    assert_eq!(f.parameters.len(), x.len(), "Parameter count mismatch");
                    for i in 0..x.len() {
                        assert_eq!(&f.parameters[i], x[i], "Parameter {}", i);
                    }
                }
                _ => panic!(format!("{:?}", $input)),
            }
        };
    }

    #[test]
    fn test_fn() {
        assert_function_declaration!("DECLARE FUNCTION Fib! (N!)", "Fib!", vec!["N!"]);
    }

    #[test]
    fn test_lower_case() {
        assert_function_declaration!("declare function echo$(msg$)", "echo$", vec!["msg$"]);
    }
}
