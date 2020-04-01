use super::{Block, Expression, Parser, QName, Statement};
use crate::common::Result;
use std::io::BufRead;

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    pub variable_name: QName,
    pub lower_bound: Expression,
    pub upper_bound: Expression,
    pub step: Option<Expression>,
    pub statements: Block,
    pub next_counter: Option<QName>,
}

impl<T: BufRead> Parser<T> {
    pub fn try_parse_for_loop(&mut self) -> Result<Option<Statement>> {
        if self.buf_lexer.try_consume_word("FOR")? {
            self.buf_lexer.demand_whitespace()?;
            let for_counter_variable = self.demand_name_with_type_qualifier()?;
            self.buf_lexer.skip_whitespace()?;
            self.buf_lexer.demand_symbol('=')?;
            self.buf_lexer.skip_whitespace()?;
            let lower_bound = self.demand_expression()?;
            self.buf_lexer.demand_whitespace()?;
            self.buf_lexer.demand_specific_word("TO")?;
            self.buf_lexer.demand_whitespace()?;
            let upper_bound = self.demand_expression()?;

            let optional_step = if self.buf_lexer.skip_whitespace()? {
                // might have "STEP" keyword
                if self.buf_lexer.try_consume_word("STEP")? {
                    self.buf_lexer.demand_whitespace()?;
                    Some(self.demand_expression()?)
                } else {
                    None
                }
            } else {
                None
            };

            self.buf_lexer.skip_whitespace()?;
            self.buf_lexer.demand_eol()?;
            self.buf_lexer.skip_whitespace_and_eol()?;

            let mut statements: Vec<Statement> = vec![];

            // might have a dummy empty for loop
            while !self.buf_lexer.try_consume_word("NEXT")? {
                statements.push(self.demand_statement()?);
                self.buf_lexer.skip_whitespace_and_eol()?;
            }

            // we are past the "NEXT", maybe there is a variable name e.g. NEXT I
            let next_counter = if self.buf_lexer.skip_whitespace()? {
                self.try_parse_name_with_type_qualifier()?
            } else {
                None
            };

            // TODO support "NEXT FOR"
            self.buf_lexer.demand_eol_or_eof()?;

            Ok(Some(Statement::ForLoop(ForLoop {
                variable_name: for_counter_variable,
                lower_bound,
                upper_bound,
                step: optional_step,
                statements,
                next_counter,
            })))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;

    #[test]
    fn test_for_loop() {
        let input = "FOR I = 1 TO 10\r\nPRINT I\r\nNEXT";
        let result = parse(input);
        assert_eq!(
            result,
            vec![top_for_loop(
                "I",
                1,
                10,
                vec![sub_call(
                    "PRINT",
                    vec![Expression::variable_name_unqualified("I")]
                )]
            )]
        );
    }

    #[test]
    fn fn_fixture_for_print_10() {
        let result = parse_file("FOR_PRINT_10.BAS");
        assert_eq!(
            result,
            vec![top_for_loop(
                "I",
                1,
                10,
                vec![sub_call(
                    "PRINT",
                    vec![
                        Expression::from("Hello"),
                        Expression::variable_name_unqualified("I")
                    ]
                )]
            )]
        );
    }

    #[test]
    fn fn_fixture_for_nested() {
        let result = parse_file("FOR_NESTED.BAS");
        assert_eq!(
            result,
            vec![
                top_sub_call("PRINT", vec![Expression::from("Before the outer loop")]),
                top_for_loop(
                    "I",
                    1,
                    10,
                    vec![
                        sub_call(
                            "PRINT",
                            vec![
                                Expression::from("Before the inner loop"),
                                Expression::variable_name_unqualified("I")
                            ]
                        ),
                        for_loop(
                            "J",
                            1,
                            10,
                            vec![sub_call(
                                "PRINT",
                                vec![
                                    Expression::from("Inner loop"),
                                    Expression::variable_name_unqualified("I"),
                                    Expression::variable_name_unqualified("J"),
                                ]
                            ),]
                        ),
                        sub_call(
                            "PRINT",
                            vec![
                                Expression::from("After the inner loop"),
                                Expression::variable_name_unqualified("I")
                            ]
                        ),
                    ]
                ),
                top_sub_call("PRINT", vec![Expression::from("After the outer loop")]),
            ]
        );
    }
}
