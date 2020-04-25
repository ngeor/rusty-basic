use super::{Parser, ParserError, StatementNode};
use crate::common::Location;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_const(&mut self, pos: Location) -> Result<StatementNode, ParserError> {
        self.read_demand_whitespace("Expected whitespace after CONST")?;
        let name_node = self.read_demand_name_node("Expected CONST name")?;
        self.read_demand_symbol_skipping_whitespace('=')?;
        let right_side = self.read_demand_expression_skipping_whitespace()?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;
        Ok(StatementNode::Const(name_node, right_side, pos))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::Location;
    use crate::parser::{StatementNode, TopLevelTokenNode};

    #[test]
    fn parse_const() {
        let input = r#"
        CONST X = 42
        CONST Y$ = "hello"
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelTokenNode::Statement(StatementNode::Const(
                    "X".as_name(2, 15),
                    42.as_lit_expr(2, 19),
                    Location::new(2, 9)
                )),
                TopLevelTokenNode::Statement(StatementNode::Const(
                    "Y$".as_name(3, 15),
                    "hello".as_lit_expr(3, 20),
                    Location::new(3, 9)
                ))
            ]
        );
    }
}
