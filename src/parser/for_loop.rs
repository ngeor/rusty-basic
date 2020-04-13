use super::{BlockNode, ForLoopNode, Parser, StatementNode};
use crate::lexer::{Keyword, LexerError};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_for_loop(&mut self) -> Result<Option<StatementNode>, LexerError> {
        let opt_pos = self.buf_lexer.try_consume_keyword(Keyword::For)?;
        if let Some(pos) = opt_pos {
            self.buf_lexer.demand_whitespace()?;
            let for_counter_variable = self.demand_name_with_type_qualifier()?;
            self.buf_lexer.skip_whitespace()?;
            self.buf_lexer.demand_symbol('=')?;
            self.buf_lexer.skip_whitespace()?;
            let lower_bound = self.demand_expression()?;
            self.buf_lexer.demand_whitespace()?;
            self.buf_lexer.demand_keyword(Keyword::To)?;
            self.buf_lexer.demand_whitespace()?;
            let upper_bound = self.demand_expression()?;

            let optional_step = if self.buf_lexer.skip_whitespace()? {
                // might have "STEP" keyword
                if self.buf_lexer.try_consume_keyword(Keyword::Step)?.is_some() {
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

            let mut statements: BlockNode = vec![];

            // might have a dummy empty for loop
            while self.buf_lexer.try_consume_keyword(Keyword::Next)?.is_none() {
                statements.push(self.demand_statement()?);
                self.buf_lexer.skip_whitespace_and_eol()?;
            }

            // we are past the "NEXT", maybe there is a variable name e.g. NEXT I
            let next_counter = if self.buf_lexer.skip_whitespace()? {
                self.try_parse_name_with_type_qualifier()?
            } else {
                None
            };

            self.buf_lexer.demand_eol_or_eof()?;

            Ok(Some(StatementNode::ForLoop(ForLoopNode {
                variable_name: for_counter_variable,
                lower_bound,
                upper_bound,
                step: optional_step,
                statements,
                next_counter,
                pos,
            })))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::parser::Expression;

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
                vec![sub_call("PRINT", vec![Expression::variable_name("I")],)],
            )],
        );
    }

    #[test]
    fn test_for_loop_lower_case() {
        let input = "for i = 1 TO 10\r\nprint i\r\nnext";
        let result = parse(input);
        assert_eq!(
            result,
            vec![top_for_loop(
                "i",
                1,
                10,
                vec![sub_call("print", vec![Expression::variable_name("i")],)],
            )],
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
                    vec![Expression::from("Hello"), Expression::variable_name("I"),],
                )],
            )],
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
                                Expression::variable_name("I"),
                            ],
                        ),
                        for_loop(
                            "J",
                            1,
                            10,
                            vec![sub_call(
                                "PRINT",
                                vec![
                                    Expression::from("Inner loop"),
                                    Expression::variable_name("I"),
                                    Expression::variable_name("J"),
                                ],
                            )],
                        ),
                        sub_call(
                            "PRINT",
                            vec![
                                Expression::from("After the inner loop"),
                                Expression::variable_name("I"),
                            ],
                        ),
                    ],
                ),
                top_sub_call("PRINT", vec![Expression::from("After the outer loop")]),
            ],
        );
    }
}
