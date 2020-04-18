use super::{
    unexpected, FunctionDeclarationNode, NameNode, Parser, ParserError, TopLevelTokenNode,
};
use crate::common::Location;
use crate::lexer::{Keyword, LexemeNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_declaration(
        &mut self,
        declare_pos: Location,
    ) -> Result<TopLevelTokenNode, ParserError> {
        self.read_demand_whitespace("Expected whitespace after DECLARE keyword")?;
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Keyword(Keyword::Function, _, _) => {
                self.read_demand_whitespace("Expected whitespace after FUNCTION keyword")?;
                let function_name =
                    self.read_demand_name_with_type_qualifier("Expected function name")?;
                let parameters: Vec<NameNode> = self.parse_declaration_parameters()?;
                Ok(TopLevelTokenNode::FunctionDeclaration(
                    FunctionDeclarationNode::new(function_name, parameters, declare_pos),
                ))
            }
            _ => unexpected("Unknown declaration", next),
        }
    }

    pub fn parse_declaration_parameters(&mut self) -> Result<Vec<NameNode>, ParserError> {
        let mut params: Vec<NameNode> = vec![];
        let next = self.read_skipping_whitespace()?;
        if next.is_symbol('(') {
            self.parse_inside_parentheses(&mut params)?;
            self.read_demand_eol_or_eof_skipping_whitespace()?;
            Ok(params)
        } else if next.is_eol_or_eof() {
            // no parentheses e.g. DECLARE FUNCTION hello
            Ok(params)
        } else {
            unexpected("Expected ( or EOL or EOF after function name", next)
        }
    }

    fn parse_inside_parentheses(&mut self, params: &mut Vec<NameNode>) -> Result<(), ParserError> {
        // holds the previous token, which can be one of:
        // '(' -> opening parenthesis (the starting point)
        // 'p' -> parameter
        // ',' -> comma
        let mut prev = '(';
        let mut found_close_parenthesis = false;
        while !found_close_parenthesis {
            let next = self.read_skipping_whitespace()?;
            match next {
                LexemeNode::Symbol(')', _) => {
                    if prev == ',' {
                        return unexpected("Expected parameter after comma", next);
                    } else {
                        found_close_parenthesis = true;
                    }
                }
                LexemeNode::Symbol(',', _) => {
                    if prev == 'p' {
                        prev = ',';
                    } else {
                        return unexpected("Unexpected comma", next);
                    }
                }
                LexemeNode::Word(_, _) => {
                    if prev == '(' || prev == ',' {
                        params.push(
                            self.demand_name_with_type_qualifier(next, "Expected parameter")?,
                        );
                        prev = 'p';
                    } else {
                        return unexpected("Unexpected name", next);
                    }
                }
                _ => {
                    return unexpected("Syntax error", next);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;

    macro_rules! assert_function_declaration {
        ($input:expr, $expected_function_name:expr, $expected_params:expr) => {
            match parse($input).demand_single() {
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
