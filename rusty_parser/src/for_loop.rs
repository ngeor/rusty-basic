use crate::expression;
use crate::pc::*;
use crate::pc_specific::*;
use crate::statements::ZeroOrMoreStatements;
use crate::types::*;
use rusty_common::*;

// FOR I = 0 TO 5 STEP 1
// statements
// NEXT (I)

pub fn for_loop_p() -> impl Parser<Output = Statement> {
    seq4(
        parse_for_step_p(),
        ZeroOrMoreStatements::new(keyword(Keyword::Next)),
        keyword(Keyword::Next).or_fail(QError::ForWithoutNext),
        next_counter_p().allow_none(),
        |(variable_name, lower_bound, upper_bound, opt_step), statements, _, opt_next_name_node| {
            Statement::ForLoop(ForLoopNode {
                variable_name,
                lower_bound,
                upper_bound,
                step: opt_step,
                statements,
                next_counter: opt_next_name_node,
            })
        },
    )
}

/// Parses the "FOR I = 1 TO 2 [STEP X]" part
fn parse_for_step_p() -> impl Parser<
    Output = (
        ExpressionNode,
        ExpressionNode,
        ExpressionNode,
        Option<ExpressionNode>,
    ),
> {
    opt_second_expression_after_keyword(parse_for_p(), Keyword::Step)
        .map(|((n, l, u), opt_step)| (n, l, u, opt_step))
}

impl ExtractExpression for (ExpressionNode, ExpressionNode, ExpressionNode) {
    fn to_expression(&self) -> &ExpressionNode {
        &self.2
    }
}

/// Parses the "FOR I = 1 TO 2" part
fn parse_for_p() -> impl Parser<Output = (ExpressionNode, ExpressionNode, ExpressionNode)> {
    seq6(
        keyword_followed_by_whitespace_p(Keyword::For),
        expression::property::parser().or_syntax_error("Expected: name after FOR"),
        equal_sign().no_incomplete(),
        expression::expr_node_ws().or_syntax_error("Expected: lower bound of FOR loop"),
        keyword(Keyword::To).no_incomplete(),
        expression::ws_expr_node().or_syntax_error("Expected: upper bound of FOR loop"),
        |_, name, _, lower_bound, _, upper_bound| (name, lower_bound, upper_bound),
    )
}

fn next_counter_p() -> impl Parser<Output = ExpressionNode> {
    whitespace()
        .and(expression::property::parser())
        .keep_right()
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_parser_err;
    use crate::*;
    use rusty_common::*;

    #[test]
    fn test_for_loop() {
        let input = "FOR I = 1 TO 10\r\nFlint I\r\nNEXT";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            Statement::ForLoop(ForLoopNode {
                variable_name: Expression::var_unresolved("I").at_rc(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![
                    Statement::SubCall("Flint".into(), vec!["I".as_var_expr(2, 7)]).at_rc(2, 1)
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
            Statement::ForLoop(ForLoopNode {
                variable_name: Expression::var_unresolved("i").at_rc(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![
                    Statement::SubCall("flint".into(), vec!["i".as_var_expr(2, 7)]).at_rc(2, 1)
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
                variable_name: Expression::var_unresolved("I").at_rc(1, 5),
                lower_bound: 1.as_lit_expr(1, 9),
                upper_bound: 10.as_lit_expr(1, 14),
                step: None,
                statements: vec![Statement::Print(PrintNode {
                    file_number: None,
                    lpt1: false,
                    format_string: None,
                    args: vec![
                        PrintArg::Expression("Hello".as_lit_expr(2, 11)),
                        PrintArg::Comma,
                        PrintArg::Expression("I".as_var_expr(2, 20))
                    ],
                })
                .at_rc(2, 5)],
                next_counter: None,
            })
        );
    }

    #[test]
    fn fn_fixture_for_nested() {
        let result = parse_file_no_location("FOR_NESTED.BAS");
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::Print(PrintNode::one(
                    "Before the outer loop".as_lit_expr(1, 7)
                ))),
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: Expression::var_unresolved("I").at_rc(2, 5),
                    lower_bound: 1.as_lit_expr(2, 9),
                    upper_bound: 10.as_lit_expr(2, 14),
                    step: None,
                    statements: vec![
                        Statement::Print(PrintNode {
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
                        Statement::ForLoop(ForLoopNode {
                            variable_name: Expression::var_unresolved("J").at_rc(4, 9),
                            lower_bound: 1.as_lit_expr(4, 13),
                            upper_bound: 10.as_lit_expr(4, 18),
                            step: None,
                            statements: vec![Statement::Print(PrintNode {
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
                            .at_rc(5, 9)],
                            next_counter: None,
                        })
                        .at_rc(4, 5),
                        Statement::Print(PrintNode {
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
                TopLevelToken::Statement(Statement::Print(PrintNode::one(
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
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: Expression::var_unresolved("I").at_rc(2, 13),
                    lower_bound: 1.as_lit_expr(2, 17),
                    upper_bound: 10.as_lit_expr(2, 22),
                    step: None,
                    statements: vec![
                        Statement::Comment(" for loop".to_string()).at_rc(2, 25),
                        Statement::SubCall("Flint".into(), vec!["I".as_var_expr(3, 15)])
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

    #[test]
    fn test_no_space_before_step() {
        let input = "
        FOR I = 0 TO 2STEP 1
        NEXT I
        ";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: end-of-statement"),
            2,
            23
        );
    }
}
