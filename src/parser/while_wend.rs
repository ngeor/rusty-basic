use super::{ConditionalBlockNode, Parser, ParserError, Statement};
use crate::lexer::Keyword;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_while_block(&mut self) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected whitespace after WHILE keyword")?;
        let condition = self.read_demand_expression()?;
        self.read_demand_eol_skipping_whitespace()?;
        let (statements, _) =
            self.parse_statements(|x| x.is_keyword(Keyword::Wend), "While without Wend")?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;
        Ok(Statement::While(ConditionalBlockNode {
            condition,
            statements,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::{BareName, ConditionalBlockNode, Expression, Operand, Statement};

    #[test]
    fn test_while_wend() {
        let input = "
        WHILE A < 5
            SYSTEM
        WEND
        ";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::While(ConditionalBlockNode {
                condition: Expression::BinaryExpression(
                    Operand::Less,
                    Box::new("A".as_var_expr(2, 15)),
                    Box::new(5.as_lit_expr(2, 19))
                )
                .at_rc(2, 17),
                statements: vec![Statement::SubCall(BareName::from("SYSTEM"), vec![]).at_rc(3, 13)]
            })
        );
    }
}
