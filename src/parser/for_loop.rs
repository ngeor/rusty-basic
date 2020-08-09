use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;

use crate::parser::expression;
use crate::parser::name;
use crate::parser::statements::parse_statements;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, QErrorNode> {
    if !lexer.peek()?.as_ref().is_keyword(Keyword::For) {
        return Ok(None);
    }

    let pos = lexer.read()?.pos();
    read_demand_whitespace(lexer, "Expected whitespace after FOR keyword")?;
    let for_counter_variable = demand(lexer, name::try_read, "Expected FOR counter variable")?;
    read_demand_symbol_skipping_whitespace(lexer, '=')?;
    let lower_bound =
        demand_skipping_whitespace(lexer, expression::try_read, "Expected lower bound")?;
    read_demand_whitespace(lexer, "Expected whitespace before TO keyword")?;
    read_demand_keyword(lexer, Keyword::To)?;
    read_demand_whitespace(lexer, "Expected whitespace after TO keyword")?;
    let upper_bound = demand(lexer, expression::try_read, "Expected upper bound")?;
    let optional_step = try_parse_step(lexer)?;

    let statements = parse_statements(lexer, |x| x.is_keyword(Keyword::Next), "FOR without NEXT")?;
    read_demand_keyword(lexer, Keyword::Next)?;

    // we are past the "NEXT", maybe there is a variable name e.g. NEXT I
    let next_counter = try_parse_next_counter(lexer)?;

    Ok(Some(
        Statement::ForLoop(ForLoopNode {
            variable_name: for_counter_variable,
            lower_bound,
            upper_bound,
            step: optional_step,
            statements,
            next_counter,
        })
        .at(pos),
    ))
}

fn try_parse_step<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<ExpressionNode>, QErrorNode> {
    const STATE_UPPER_BOUND: u8 = 0;
    const STATE_WHITESPACE_BEFORE_STEP: u8 = 1;
    const STATE_STEP: u8 = 2;
    const STATE_WHITESPACE_AFTER_STEP: u8 = 3;
    const STATE_STEP_EXPR: u8 = 4;
    const STATE_WHITESPACE_BEFORE_EOL: u8 = 5;
    const STATE_EOL: u8 = 6;
    let mut state = STATE_UPPER_BOUND;
    let mut expr: Option<ExpressionNode> = None;

    lexer.begin_transaction();

    while state != STATE_EOL {
        let next = lexer.peek()?;
        match next.as_ref() {
            Lexeme::Whitespace(_) => {
                lexer.read()?;
                if state == STATE_UPPER_BOUND {
                    state = STATE_WHITESPACE_BEFORE_STEP;
                } else if state == STATE_STEP {
                    state = STATE_WHITESPACE_AFTER_STEP;
                } else if state == STATE_STEP_EXPR {
                    state = STATE_WHITESPACE_BEFORE_EOL;
                } else {
                    return Err(QError::SyntaxError("Unexpected whitespace".to_string()))
                        .with_err_at(&next);
                }
            }
            Lexeme::EOF => {
                return Err(QError::SyntaxError("FOR without NEXT".to_string())).with_err_at(&next)
            }
            Lexeme::EOL(_) => {
                if state == STATE_STEP || state == STATE_WHITESPACE_AFTER_STEP {
                    return Err(QError::SyntaxError(
                        "Expected expression after STEP".to_string(),
                    ))
                    .with_err_at(&next);
                }
                state = STATE_EOL;
            }
            Lexeme::Keyword(Keyword::Step, _) => {
                lexer.read()?;
                if state == STATE_WHITESPACE_BEFORE_STEP {
                    state = STATE_STEP;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(&next);
                }
            }
            _ => {
                if state == STATE_UPPER_BOUND || state == STATE_WHITESPACE_BEFORE_STEP {
                    // bail out, we didn't find the STEP keyword but something else (maybe a comment?)
                    state = STATE_EOL;
                } else if state == STATE_WHITESPACE_AFTER_STEP {
                    expr = Some(demand(
                        lexer,
                        expression::try_read,
                        "Expected expression after STEP",
                    )?);
                    state = STATE_STEP_EXPR;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(&next);
                }
            }
        }
    }
    lexer.commit_transaction();
    Ok(expr)
}

fn try_parse_next_counter<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<NameNode>, QErrorNode> {
    const STATE_NEXT: u8 = 0;
    const STATE_WHITESPACE_AFTER_NEXT: u8 = 1;
    const STATE_EOL_OR_EOF: u8 = 2;
    let mut state = STATE_NEXT;
    let mut name: Option<NameNode> = None;

    lexer.begin_transaction();

    while state != STATE_EOL_OR_EOF {
        let next = lexer.peek()?;
        match next.as_ref() {
            Lexeme::Whitespace(_) => {
                lexer.read()?;
                if state == STATE_NEXT {
                    state = STATE_WHITESPACE_AFTER_NEXT;
                }
            }
            Lexeme::EOL(_) | Lexeme::EOF => {
                state = STATE_EOL_OR_EOF;
            }
            Lexeme::Word(_) => {
                if state == STATE_WHITESPACE_AFTER_NEXT {
                    name = Some(demand(
                        lexer,
                        name::try_read,
                        "Expected NEXT counter variable",
                    )?);
                    state = STATE_EOL_OR_EOF;
                } else {
                    return Err(QError::SyntaxError("Syntax error".to_string())).with_err_at(&next);
                }
            }
            _ => {
                state = STATE_EOL_OR_EOF;
            }
        }
    }

    lexer.commit_transaction();

    Ok(name)
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

    #[test]
    fn test_inline_comment() {
        let input = r#"
        FOR I = 1 TO 10 ' for loop
        PRINT I ' print it
        NEXT ' end of loop
        "#;
        let result = parse(input);
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: "I".as_name(2, 13),
                    lower_bound: 1.as_lit_expr(2, 17),
                    upper_bound: 10.as_lit_expr(2, 22),
                    step: None,
                    statements: vec![
                        Statement::Comment(" for loop".to_string()).at_rc(2, 25),
                        Statement::SubCall("PRINT".into(), vec!["I".as_var_expr(3, 15)])
                            .at_rc(3, 9),
                        Statement::Comment(" print it".to_string()).at_rc(3, 17),
                    ],
                    next_counter: None,
                }))
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" end of loop".to_string()))
                    .at_rc(4, 14)
            ]
        );
    }
}
