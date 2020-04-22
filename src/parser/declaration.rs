use super::{unexpected, NameNode, Parser, ParserError, TopLevelTokenNode};
use crate::common::Location;
use crate::lexer::{Keyword, LexemeNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_declaration(&mut self, pos: Location) -> Result<TopLevelTokenNode, ParserError> {
        self.read_demand_whitespace("Expected whitespace after DECLARE keyword")?;
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Keyword(Keyword::Function, _, _) => {
                self.read_demand_whitespace("Expected whitespace after FUNCTION keyword")?;
                let function_name = self.read_demand_name_node("Expected function name")?;
                let parameters = self.parse_declaration_parameters()?;
                Ok(TopLevelTokenNode::FunctionDeclaration(
                    function_name,
                    parameters,
                    pos,
                ))
            }
            LexemeNode::Keyword(Keyword::Sub, _, _) => {
                self.read_demand_whitespace("Expected whitespace after SUB keyword")?;
                let sub_name = self.read_demand_bare_name_node("Expected sub name")?;
                let parameters = self.parse_declaration_parameters()?;
                Ok(TopLevelTokenNode::SubDeclaration(sub_name, parameters, pos))
            }
            _ => unexpected("Unknown declaration", next),
        }
    }

    pub fn demand_function_implementation(
        &mut self,
        pos: Location,
    ) -> Result<TopLevelTokenNode, ParserError> {
        // function name
        self.read_demand_whitespace("Expected whitespace after FUNCTION keyword")?;
        let name = self.read_demand_name_node("Expected function name")?;
        // function parameters
        let params: Vec<NameNode> = self.parse_declaration_parameters()?;
        // function body
        let (block, _) =
            self.parse_statements(|x| x.is_keyword(Keyword::End), "Function without End")?;
        self.read_demand_whitespace("Expected whitespace after END keyword")?;
        self.read_demand_keyword(Keyword::Function)?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;

        Ok(TopLevelTokenNode::FunctionImplementation(
            name, params, block, pos,
        ))
    }

    pub fn demand_sub_implementation(
        &mut self,
        pos: Location,
    ) -> Result<TopLevelTokenNode, ParserError> {
        // sub name
        self.read_demand_whitespace("Expected whitespace after SUB keyword")?;
        let name = self.read_demand_bare_name_node("Expected sub name")?;
        // sub parameters
        let params: Vec<NameNode> = self.parse_declaration_parameters()?;
        // body
        let (block, _) =
            self.parse_statements(|x| x.is_keyword(Keyword::End), "Sub without End")?;
        self.read_demand_whitespace("Expected whitespace after END keyword")?;
        self.read_demand_keyword(Keyword::Sub)?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;
        Ok(TopLevelTokenNode::SubImplementation(
            name, params, block, pos,
        ))
    }

    fn parse_declaration_parameters(&mut self) -> Result<Vec<NameNode>, ParserError> {
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
                        params.push(self.demand_name_node(next, "Expected parameter")?);
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
    use crate::common::Location;
    use crate::parser::{ExpressionNode, Operand, OperandNode, StatementNode, TopLevelTokenNode};

    macro_rules! assert_function_declaration {
        ($input:expr, $expected_function_name:expr, $expected_params:expr) => {
            match parse($input).demand_single() {
                TopLevelTokenNode::FunctionDeclaration(name, parameters, _) => {
                    assert_eq!(&name, $expected_function_name, "Function name mismatch");
                    let x = $expected_params;
                    assert_eq!(parameters.len(), x.len(), "Parameter count mismatch");
                    for i in 0..x.len() {
                        assert_eq!(&parameters[i], x[i], "Parameter {}", i);
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

    #[test]
    fn test_function_implementation() {
        let input = "
        FUNCTION Add(A, B)
            Add = A + B
        END FUNCTION
        ";
        let result = parse(input).demand_single();
        assert_eq!(
            result,
            TopLevelTokenNode::FunctionImplementation(
                "Add".as_name(2, 18),
                vec!["A".as_name(2, 22), "B".as_name(2, 25)],
                vec![StatementNode::Assignment(
                    "Add".as_name(3, 13),
                    ExpressionNode::BinaryExpression(
                        OperandNode::new(Operand::Plus, Location::new(3, 21)),
                        Box::new("A".as_var_expr(3, 19)),
                        Box::new("B".as_var_expr(3, 23))
                    )
                )],
                Location::new(2, 9)
            )
        );
    }

    #[test]
    fn test_function_implementation_lower_case() {
        let input = "
        function add(a, b)
            add = a + b
        end function
        ";
        let result = parse(input).demand_single();
        assert_eq!(
            result,
            TopLevelTokenNode::FunctionImplementation(
                "add".as_name(2, 18),
                vec!["a".as_name(2, 22), "b".as_name(2, 25)],
                vec![StatementNode::Assignment(
                    "add".as_name(3, 13),
                    ExpressionNode::BinaryExpression(
                        OperandNode::new(Operand::Plus, Location::new(3, 21)),
                        Box::new("a".as_var_expr(3, 19)),
                        Box::new("b".as_var_expr(3, 23))
                    )
                )],
                Location::new(2, 9)
            )
        );
    }
}
