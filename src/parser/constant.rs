use super::{Parser, ParserError, Statement};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_const(&mut self) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected whitespace after CONST")?;
        let name_node = self.read_demand_name_node("Expected CONST name")?;
        self.read_demand_symbol_skipping_whitespace('=')?;
        let right_side = self.read_demand_expression_skipping_whitespace()?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;
        Ok(Statement::Const(name_node, right_side))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Statement, TopLevelToken};

    #[test]
    fn parse_const() {
        let input = r#"
        CONST X = 42
        CONST Y$ = "hello"
        "#;
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Const(
                    "X".as_name(2, 15),
                    42.as_lit_expr(2, 19),
                )),
                TopLevelToken::Statement(Statement::Const(
                    "Y$".as_name(3, 15),
                    "hello".as_lit_expr(3, 20),
                ))
            ]
        );
    }
}
