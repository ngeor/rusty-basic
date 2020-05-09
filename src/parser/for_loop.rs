use super::{unexpected, ExpressionNode, ForLoopNode, NameNode, Parser, ParserError, Statement};
use crate::lexer::{Keyword, LexemeNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_for_loop(&mut self) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected whitespace after FOR keyword")?;
        let for_counter_variable = self.read_demand_name_node("Expected FOR counter variable")?;
        self.read_demand_symbol_skipping_whitespace('=')?;
        let lower_bound = self.read_demand_expression_skipping_whitespace()?;
        self.read_demand_whitespace("Expected whitespace before TO keyword")?;
        self.read_demand_keyword(Keyword::To)?;
        self.read_demand_whitespace("Expected whitespace after TO keyword")?;
        let upper_bound = self.read_demand_expression()?;
        let optional_step = self.try_parse_step()?;

        let (statements, _) =
            self.parse_statements(|x| x.is_keyword(Keyword::Next), "FOR without NEXT")?;

        // we are past the "NEXT", maybe there is a variable name e.g. NEXT I
        let next_counter = self.try_parse_next_counter()?;

        Ok(Statement::ForLoop(ForLoopNode {
            variable_name: for_counter_variable,
            lower_bound,
            upper_bound,
            step: optional_step,
            statements,
            next_counter,
        }))
    }

    fn try_parse_step(&mut self) -> Result<Option<ExpressionNode>, ParserError> {
        const STATE_UPPER_BOUND: u8 = 0;
        const STATE_WHITESPACE_BEFORE_STEP: u8 = 1;
        const STATE_STEP: u8 = 2;
        const STATE_WHITESPACE_AFTER_STEP: u8 = 3;
        const STATE_STEP_EXPR: u8 = 4;
        const STATE_WHITESPACE_BEFORE_EOL: u8 = 5;
        const STATE_EOL: u8 = 6;
        let mut state = STATE_UPPER_BOUND;
        let mut expr: Option<ExpressionNode> = None;
        while state != STATE_EOL {
            let next = self.buf_lexer.read()?;
            match next {
                LexemeNode::Whitespace(_, _) => {
                    if state == STATE_UPPER_BOUND {
                        state = STATE_WHITESPACE_BEFORE_STEP;
                    } else if state == STATE_STEP {
                        state = STATE_WHITESPACE_AFTER_STEP;
                    } else if state == STATE_STEP_EXPR {
                        state = STATE_WHITESPACE_BEFORE_EOL;
                    } else {
                        return unexpected("Unexpected whitespace", next);
                    }
                }
                LexemeNode::EOF(_) => return unexpected("FOR without NEXT", next),
                LexemeNode::EOL(_, _) => {
                    if state == STATE_STEP || state == STATE_WHITESPACE_AFTER_STEP {
                        return unexpected("Expected expression after STEP", next);
                    }
                    state = STATE_EOL;
                }
                LexemeNode::Keyword(Keyword::Step, _, _) => {
                    if state == STATE_WHITESPACE_BEFORE_STEP {
                        state = STATE_STEP;
                    } else {
                        return unexpected("Syntax error", next);
                    }
                }
                _ => {
                    if state == STATE_WHITESPACE_AFTER_STEP {
                        expr = Some(self.demand_expression(next)?);
                        state = STATE_STEP_EXPR;
                    } else {
                        return unexpected("Syntax error", next);
                    }
                }
            }
        }
        Ok(expr)
    }

    fn try_parse_next_counter(&mut self) -> Result<Option<NameNode>, ParserError> {
        const STATE_NEXT: u8 = 0;
        const STATE_WHITESPACE_AFTER_NEXT: u8 = 1;
        const STATE_EOL_OR_EOF: u8 = 2;
        const STATE_WORD: u8 = 3;
        let mut state = STATE_NEXT;
        let mut name: Option<NameNode> = None;
        while state != STATE_EOL_OR_EOF {
            let next = self.buf_lexer.read()?;
            match next {
                LexemeNode::Whitespace(_, _) => {
                    if state == STATE_NEXT {
                        state = STATE_WHITESPACE_AFTER_NEXT;
                    }
                }
                LexemeNode::EOL(_, _) | LexemeNode::EOF(_) => {
                    state = STATE_EOL_OR_EOF;
                }
                LexemeNode::Word(_, _) => {
                    if state == STATE_WHITESPACE_AFTER_NEXT {
                        name = Some(self.demand_name_node(next, "Expected NEXT counter variable")?);
                        state = STATE_WORD;
                    } else {
                        return unexpected("Syntax error", next);
                    }
                }
                _ => return unexpected("Expected variable or EOL or EOF", next),
            }
        }
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::*;

    #[test]
    fn test_for_loop() {
        let input = "FOR I = 1 TO 10\r\nPRINT I\r\nNEXT";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            Statement::ForLoop(ForLoopNode {
                variable_name: "I".as_name(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![
                    Statement::SubCall("PRINT".into(), vec!["I".as_var_expr(2, 7)]).at_rc(2, 1)
                ],
                next_counter: None,
            })
        );
    }

    #[test]
    fn test_for_loop_lower_case() {
        let input = "for i = 1 TO 10\r\nprint i\r\nnext";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            Statement::ForLoop(ForLoopNode {
                variable_name: "i".as_name(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![
                    Statement::SubCall("print".into(), vec!["i".as_var_expr(2, 7)]).at_rc(2, 1)
                ],
                next_counter: None,
            })
        );
    }

    #[test]
    fn fn_fixture_for_print_10() {
        let result = parse_file("FOR_PRINT_10.BAS").demand_single_statement();
        assert_eq!(
            result,
            Statement::ForLoop(ForLoopNode {
                variable_name: "I".as_name(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![Statement::SubCall(
                    "PRINT".into(),
                    vec!["Hello".as_lit_expr(2, 11), "I".as_var_expr(2, 20)]
                )
                .at_rc(2, 5)],
                next_counter: None,
            })
        );
    }

    #[test]
    fn fn_fixture_for_nested() {
        let result = parse_file("FOR_NESTED.BAS").strip_location();
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::SubCall(
                    "PRINT".into(),
                    vec!["Before the outer loop".as_lit_expr(1, 7)]
                )),
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: "I".as_name(2, 5),
                    lower_bound: 1.as_lit_expr(2, 9),
                    upper_bound: 10.as_lit_expr(2, 14),
                    step: None,
                    statements: vec![
                        Statement::SubCall(
                            "PRINT".into(),
                            vec![
                                "Before the inner loop".as_lit_expr(3, 11),
                                "I".as_var_expr(3, 36)
                            ]
                        )
                        .at_rc(3, 5),
                        Statement::ForLoop(ForLoopNode {
                            variable_name: "J".as_name(4, 9),
                            lower_bound: 1.as_lit_expr(4, 13),
                            upper_bound: 10.as_lit_expr(4, 18),
                            step: None,
                            statements: vec![Statement::SubCall(
                                "PRINT".into(),
                                vec![
                                    "Inner loop".as_lit_expr(5, 15),
                                    "I".as_var_expr(5, 29),
                                    "J".as_var_expr(5, 32)
                                ]
                            )
                            .at_rc(5, 9)],
                            next_counter: None,
                        })
                        .at_rc(4, 5),
                        Statement::SubCall(
                            "PRINT".into(),
                            vec![
                                "After the inner loop".as_lit_expr(7, 11),
                                "I".as_var_expr(7, 35)
                            ]
                        )
                        .at_rc(7, 5)
                    ],
                    next_counter: None,
                })),
                TopLevelToken::Statement(Statement::SubCall(
                    BareName::from("PRINT"),
                    vec!["After the outer loop".as_lit_expr(9, 7)]
                )),
            ]
        );
    }
}
