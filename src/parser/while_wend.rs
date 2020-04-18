use super::{ConditionalBlockNode, Parser, ParserError, StatementNode};
use crate::common::Location;
use crate::lexer::Keyword;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_while_block(&mut self, pos: Location) -> Result<StatementNode, ParserError> {
        self.read_demand_whitespace("Expected whitespace after WHILE keyword")?;
        let condition = self.read_demand_expression()?;
        self.read_demand_eol_skipping_whitespace()?;
        let (statements, _) =
            self.parse_statements(|x| x.is_keyword(Keyword::Wend), "While without Wend")?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;
        Ok(StatementNode::While(ConditionalBlockNode::new(
            pos, condition, statements,
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Location;
    use crate::parser::test_utils::*;
    use crate::parser::{
        ConditionalBlockNode, ExpressionNode, Operand, OperandNode, StatementNode,
        TopLevelTokenNode,
    };

    #[test]
    fn test_while_wend() {
        let input = "
        WHILE A < 5
            SYSTEM
        WEND
        ";
        let program = parse(input);
        assert_eq!(
            program,
            vec![TopLevelTokenNode::Statement(StatementNode::While(
                ConditionalBlockNode::new(
                    Location::new(2, 9),
                    ExpressionNode::BinaryExpression(
                        OperandNode::new(Operand::LessThan, Location::new(2, 17)),
                        Box::new("A".as_var_expr(2, 15)),
                        Box::new(5.as_lit_expr(2, 19))
                    ),
                    vec![StatementNode::SubCall("SYSTEM".as_name(3, 13), vec![])]
                )
            ))]
        );
    }
}
