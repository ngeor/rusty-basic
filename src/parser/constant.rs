use super::{Parser, ParserError, Statement, StatementContext};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_const(&mut self) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected whitespace after CONST")?;
        let name_node = self.read_demand_name_node("Expected CONST name")?;
        self.read_demand_symbol_skipping_whitespace('=')?;
        let right_side = self.read_demand_expression_skipping_whitespace()?;
        self.finish_line(StatementContext::Normal)?;
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

    #[test]
    fn test_inline_comment() {
        let input = "CONST ANSWER = 42 ' the answer to life, universe, everything";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Const(
                    "ANSWER".as_name(1, 7),
                    42.as_lit_expr(1, 16)
                ))
                .at_rc(1, 1),
                TopLevelToken::Statement(Statement::Comment(
                    " the answer to life, universe, everything".to_string(),
                ))
                .at_rc(1, 19)
            ]
        );
    }
}
