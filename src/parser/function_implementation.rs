use super::{FunctionImplementationNode, NameNode, Parser, ParserError, TopLevelTokenNode};
use crate::lexer::Keyword;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_function_implementation(
        &mut self,
    ) -> Result<Option<TopLevelTokenNode>, ParserError> {
        let opt_pos = self.buf_lexer.try_consume_keyword(Keyword::Function)?;
        if let Some(pos) = opt_pos {
            // function name
            self.buf_lexer.demand_whitespace()?;
            let name = self.demand_name_with_type_qualifier()?;
            // function parameters
            self.buf_lexer.skip_whitespace()?;
            let function_arguments: Vec<NameNode> = self.parse_declaration_parameters()?;
            self.buf_lexer.demand_eol_or_eof()?;
            let block = self.parse_block()?;
            self.buf_lexer.demand_keyword(Keyword::End)?;
            self.buf_lexer.demand_whitespace()?;
            self.buf_lexer.demand_keyword(Keyword::Function)?;
            self.buf_lexer.demand_eol_or_eof()?;

            Ok(Some(TopLevelTokenNode::FunctionImplementation(
                FunctionImplementationNode::new(name, function_arguments, block, pos),
            )))
        } else {
            Ok(None)
        }
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
