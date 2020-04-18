use super::{FunctionImplementationNode, NameNode, Parser, ParserError, TopLevelTokenNode};
use crate::common::Location;
use crate::lexer::Keyword;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_function_implementation(
        &mut self,
        pos: Location,
    ) -> Result<TopLevelTokenNode, ParserError> {
        // function name
        self.read_demand_whitespace("Expected whitespace after FUNCTION keyword")?;
        let name = self.read_demand_name_with_type_qualifier("Expected function name")?;
        // function parameters
        let function_arguments: Vec<NameNode> = self.parse_declaration_parameters()?;
        // function body
        let (block, _) =
            self.parse_statements(|x| x.is_keyword(Keyword::End), "Function without End")?;
        self.read_demand_whitespace("Expected whitespace after END keyword")?;
        self.read_demand_keyword(Keyword::Function)?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;

        Ok(TopLevelTokenNode::FunctionImplementation(
            FunctionImplementationNode::new(name, function_arguments, block, pos),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::common::Location;
    use crate::parser::{ExpressionNode, Operand, OperandNode, StatementNode};

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
            TopLevelTokenNode::FunctionImplementation(FunctionImplementationNode::new(
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
            ))
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
            TopLevelTokenNode::FunctionImplementation(FunctionImplementationNode::new(
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
            ))
        );
    }
}
