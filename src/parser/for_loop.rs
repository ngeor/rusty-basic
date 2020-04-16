use super::{BlockNode, ForLoopNode, Parser, ParserError, StatementNode};
use crate::lexer::Keyword;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_for_loop(&mut self) -> Result<Option<StatementNode>, ParserError> {
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
    use super::*;
    use crate::common::Location;
    use crate::parser::TopLevelTokenNode;

    #[test]
    fn test_for_loop() {
        let input = "FOR I = 1 TO 10\r\nPRINT I\r\nNEXT";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            StatementNode::ForLoop(ForLoopNode {
                variable_name: "I".as_name(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![StatementNode::SubCall(
                    "PRINT".as_name(2, 1),
                    vec!["I".as_var_expr(2, 7)]
                )],
                next_counter: None,
                pos: Location::new(1, 1)
            })
        );
    }

    #[test]
    fn test_for_loop_lower_case() {
        let input = "for i = 1 TO 10\r\nprint i\r\nnext";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            StatementNode::ForLoop(ForLoopNode {
                variable_name: "i".as_name(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![StatementNode::SubCall(
                    "print".as_name(2, 1),
                    vec!["i".as_var_expr(2, 7)]
                )],
                next_counter: None,
                pos: Location::new(1, 1)
            })
        );
    }

    #[test]
    fn fn_fixture_for_print_10() {
        let result = parse_file("FOR_PRINT_10.BAS").demand_single_statement();
        assert_eq!(
            result,
            StatementNode::ForLoop(ForLoopNode {
                variable_name: "I".as_name(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![StatementNode::SubCall(
                    "PRINT".as_name(2, 5),
                    vec!["Hello".as_lit_expr(2, 11), "I".as_var_expr(2, 20)]
                )],
                next_counter: None,
                pos: Location::new(1, 1)
            })
        );
    }

    #[test]
    fn fn_fixture_for_nested() {
        let result = parse_file("FOR_NESTED.BAS");
        assert_eq!(
            result,
            vec![
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "PRINT".as_name(1, 1),
                    vec!["Before the outer loop".as_lit_expr(1, 7)]
                )),
                TopLevelTokenNode::Statement(StatementNode::ForLoop(ForLoopNode {
                    variable_name: "I".as_name(2, 5),
                    lower_bound: 1.as_lit_expr(2, 9),
                    upper_bound: 10.as_lit_expr(2, 14),
                    step: None,
                    statements: vec![
                        StatementNode::SubCall(
                            "PRINT".as_name(3, 5),
                            vec![
                                "Before the inner loop".as_lit_expr(3, 11),
                                "I".as_var_expr(3, 36)
                            ]
                        ),
                        StatementNode::ForLoop(ForLoopNode {
                            variable_name: "J".as_name(4, 9),
                            lower_bound: 1.as_lit_expr(4, 13),
                            upper_bound: 10.as_lit_expr(4, 18),
                            step: None,
                            statements: vec![StatementNode::SubCall(
                                "PRINT".as_name(5, 9),
                                vec![
                                    "Inner loop".as_lit_expr(5, 15),
                                    "I".as_var_expr(5, 29),
                                    "J".as_var_expr(5, 32)
                                ]
                            )],
                            next_counter: None,
                            pos: Location::new(4, 5)
                        }),
                        StatementNode::SubCall(
                            "PRINT".as_name(7, 5),
                            vec![
                                "After the inner loop".as_lit_expr(7, 11),
                                "I".as_var_expr(7, 35)
                            ]
                        )
                    ],
                    next_counter: None,
                    pos: Location::new(2, 1)
                })),
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "PRINT".as_name(9, 1),
                    vec!["After the outer loop".as_lit_expr(9, 7)]
                )),
            ]
        );
    }
}
