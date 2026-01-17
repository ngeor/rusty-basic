use rusty_pc::*;

use crate::core::expression::{expr_pos_ws_p, property, ws_expr_pos_p};
use crate::core::opt_second_expression::opt_second_expression_after_keyword;
use crate::core::statements::zero_or_more_statements;
use crate::error::ParserError;
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::{equal_sign_ws, whitespace_ignoring};
use crate::*;

// FOR I = 0 TO 5 STEP 1
// statements
// NEXT (I)

pub fn for_loop_p() -> impl Parser<RcStringView, Output = Statement, Error = ParserError> {
    seq4(
        parse_for_step_p(),
        zero_or_more_statements!(Keyword::Next),
        keyword(Keyword::Next).or_fail(ParserError::fatal(ParserErrorKind::ForWithoutNext)),
        next_counter_p().to_option(),
        |(variable_name, lower_bound, upper_bound, opt_step), statements, _, opt_next_name_pos| {
            Statement::ForLoop(ForLoop {
                variable_name,
                lower_bound,
                upper_bound,
                step: opt_step,
                statements,
                next_counter: opt_next_name_pos,
            })
        },
    )
}

/// Parses the "FOR I = 1 TO 2 [STEP X]" part
fn parse_for_step_p() -> impl Parser<
    RcStringView,
    Output = (
        ExpressionPos,
        ExpressionPos,
        ExpressionPos,
        Option<ExpressionPos>,
    ),
    Error = ParserError,
> {
    opt_second_expression_after_keyword(parse_for_p(), Keyword::Step, |(_var, _low, upper)| {
        upper.is_parenthesis()
    })
    .map(|((n, l, u), opt_step)| (n, l, u, opt_step))
}

/// Parses the "FOR I = 1 TO 2" part
fn parse_for_p()
-> impl Parser<RcStringView, Output = (ExpressionPos, ExpressionPos, ExpressionPos), Error = ParserError>
{
    seq6(
        keyword_ws_p(Keyword::For),
        property::parser().or_expected("name after FOR"),
        equal_sign_ws(),
        expr_pos_ws_p().or_expected("lower bound of FOR loop"),
        keyword(Keyword::To),
        ws_expr_pos_p().or_expected("upper bound of FOR loop"),
        |_, name, _, lower_bound, _, upper_bound| (name, lower_bound, upper_bound),
    )
}

fn next_counter_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParserError> {
    whitespace_ignoring().and_keep_right(property::parser())
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::test_utils::*;
    use crate::{assert_parser_err, *};
    #[test]
    fn test_for_loop() {
        let input = "FOR I = 1 TO 10\r\nFlint I\r\nNEXT";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            Statement::ForLoop(ForLoop {
                variable_name: Expression::var_unresolved("I").at_rc(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![
                    Statement::sub_call("Flint".into(), vec!["I".as_var_expr(2, 7)]).at_rc(2, 1)
                ],
                next_counter: None,
            })
        );
    }

    #[test]
    fn test_for_loop_lower_case() {
        let input = "for i = 1 TO 10\r\nflint i\r\nnext";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            Statement::ForLoop(ForLoop {
                variable_name: Expression::var_unresolved("i").at_rc(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![
                    Statement::sub_call("Flint".into(), vec!["i".as_var_expr(2, 7)]).at_rc(2, 1)
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
            Statement::ForLoop(ForLoop {
                variable_name: Expression::var_unresolved("I").at_rc(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![
                    Statement::Print(Print {
                        file_number: None,
                        lpt1: false,
                        format_string: None,
                        args: vec![
                            PrintArg::Expression("Hello".as_lit_expr(2, 11)),
                            PrintArg::Comma,
                            PrintArg::Expression("I".as_var_expr(2, 20))
                        ],
                    })
                    .at_rc(2, 5)
                ],
                next_counter: None,
            })
        );
    }

    #[test]
    fn fn_fixture_for_nested() {
        let result = parse_file_no_pos("FOR_NESTED.BAS");
        assert_eq!(
            result,
            vec![
                GlobalStatement::Statement(Statement::Print(Print::one(
                    "Before the outer loop".as_lit_expr(1, 7)
                ))),
                GlobalStatement::Statement(Statement::ForLoop(ForLoop {
                    variable_name: Expression::var_unresolved("I").at_rc(2, 5),
                    lower_bound: 1.as_lit_expr(2, 9),
                    upper_bound: 10.as_lit_expr(2, 14),
                    step: None,
                    statements: vec![
                        Statement::Print(Print {
                            file_number: None,
                            lpt1: false,
                            format_string: None,
                            args: vec![
                                PrintArg::Expression("Before the inner loop".as_lit_expr(3, 11)),
                                PrintArg::Comma,
                                PrintArg::Expression("I".as_var_expr(3, 36))
                            ],
                        })
                        .at_rc(3, 5),
                        Statement::ForLoop(ForLoop {
                            variable_name: Expression::var_unresolved("J").at_rc(4, 9),
                            lower_bound: 1.as_lit_expr(4, 13),
                            upper_bound: 10.as_lit_expr(4, 18),
                            step: None,
                            statements: vec![
                                Statement::Print(Print {
                                    file_number: None,
                                    lpt1: false,
                                    format_string: None,
                                    args: vec![
                                        PrintArg::Expression("Inner loop".as_lit_expr(5, 15)),
                                        PrintArg::Comma,
                                        PrintArg::Expression("I".as_var_expr(5, 29)),
                                        PrintArg::Comma,
                                        PrintArg::Expression("J".as_var_expr(5, 32)),
                                    ],
                                })
                                .at_rc(5, 9)
                            ],
                            next_counter: None,
                        })
                        .at_rc(4, 5),
                        Statement::Print(Print {
                            file_number: None,
                            lpt1: false,
                            format_string: None,
                            args: vec![
                                PrintArg::Expression("After the inner loop".as_lit_expr(7, 11)),
                                PrintArg::Comma,
                                PrintArg::Expression("I".as_var_expr(7, 35))
                            ],
                        })
                        .at_rc(7, 5)
                    ],
                    next_counter: None,
                })),
                GlobalStatement::Statement(Statement::Print(Print::one(
                    "After the outer loop".as_lit_expr(9, 7)
                ))),
            ]
        );
    }

    #[test]
    fn test_inline_comment() {
        let input = r#"
        FOR I = 1 TO 10 ' for loop
        Flint I ' print it
        NEXT ' end of loop
        "#;
        let result = parse(input);
        assert_eq!(
            result,
            vec![
                GlobalStatement::Statement(Statement::ForLoop(ForLoop {
                    variable_name: Expression::var_unresolved("I").at_rc(2, 13),
                    lower_bound: 1.as_lit_expr(2, 17),
                    upper_bound: 10.as_lit_expr(2, 22),
                    step: None,
                    statements: vec![
                        Statement::Comment(" for loop".to_string()).at_rc(2, 25),
                        Statement::sub_call("Flint".into(), vec!["I".as_var_expr(3, 15)])
                            .at_rc(3, 9),
                        Statement::Comment(" print it".to_string()).at_rc(3, 17),
                    ],
                    next_counter: None,
                }))
                .at_rc(2, 9),
                GlobalStatement::Statement(Statement::Comment(" end of loop".to_string()))
                    .at_rc(4, 14)
            ]
        );
    }

    #[test]
    fn test_no_space_before_step() {
        let input = "
        FOR I = 0 TO 2STEP 1
        NEXT I
        ";
        assert_parser_err!(input, ParserErrorKind::expected("end-of-statement"), 2, 23);
    }
}
