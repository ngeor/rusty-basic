use crate::common::{HasLocation, QError};
use crate::parser::name;
use crate::parser::pc::{Reader, ReaderResult};
use crate::parser::pc2::binary::BinaryParser;
use crate::parser::pc2::text::{whitespace_p, TextParser, Whitespace};
use crate::parser::pc2::unary::UnaryParser;
use crate::parser::pc2::unary_fn::UnaryFnParser;
use crate::parser::pc2::{item_p, static_err_p, Parser};
use crate::parser::pc_specific::{keyword_p, PcSpecific};
use crate::parser::statements;
use crate::parser::{
    expression, Expression, ExpressionNode, ExpressionType, ForLoopNode, Keyword, NameNode,
    Statement,
};

// FOR I = 0 TO 5 STEP 1
// statements
// NEXT (I)

#[deprecated]
pub fn for_loop<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    for_loop_p().convert_to_fn()
}

// TODO use expression parser to parse variable names for FOR and NEXT counter

pub fn for_loop_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    parse_for_step_p()
        .and_demand(
            statements::statements_p(keyword_p(Keyword::Next))
                .or_syntax_error("Expected: STEP or end-of-statement"),
        )
        .and_demand(next_counter_p().or(static_err_p(QError::ForWithoutNext)))
        .map(
            |(
                ((variable_name, lower_bound, upper_bound, opt_step), statements),
                opt_next_name_node,
            )| {
                Statement::ForLoop(ForLoopNode {
                    variable_name: variable_name
                        .map(|x| Expression::Variable(x, ExpressionType::Unresolved)),
                    lower_bound,
                    upper_bound,
                    step: opt_step,
                    statements,
                    next_counter: opt_next_name_node
                        .map(|x| x.map(|y| Expression::Variable(y, ExpressionType::Unresolved))),
                })
            },
        )
}

/// Parses the "FOR I = 1 TO 2 [STEP X]" part
fn parse_for_step_p<R>() -> impl Parser<
    R,
    Output = (
        NameNode,
        ExpressionNode,
        ExpressionNode,
        Option<ExpressionNode>,
    ),
>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    parse_for_p()
        .and_opt_factory(|(_, _, upper)| {
            Whitespace::new(!upper.is_parenthesis())
                .and(keyword_p(Keyword::Step))
                .and_demand(
                    expression::guarded_expression_node_p()
                        .or_syntax_error("Expected: expression after STEP"),
                )
                .keep_right()
        })
        .map(|((n, l, u), opt_step)| (n, l, u, opt_step))
}

/// Parses the "FOR I = 1 TO 2" part
fn parse_for_p<R>() -> impl Parser<R, Output = (NameNode, ExpressionNode, ExpressionNode)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::For)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after FOR"))
        .and_demand(
            name::name_with_dot_p()
                .with_pos()
                .or_syntax_error("Expected: name after FOR"),
        )
        .and_demand(
            item_p('=')
                .preceded_by_opt_ws()
                .or_syntax_error("Expected: = after name"),
        )
        .and_demand(
            expression::back_guarded_expression_node_p()
                .or_syntax_error("Expected: lower bound of FOR loop"),
        )
        .and_demand(keyword_p(Keyword::To).or_syntax_error("Expected: TO"))
        .and_demand(
            expression::guarded_expression_node_p()
                .or_syntax_error("Expected: upper bound of FOR loop"),
        )
        .map(|(((((_, n), _), l), _), u)| (n, l, u))
}

fn next_counter_p<R>() -> impl Parser<R, Output = Option<NameNode>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Next)
        .and_opt(whitespace_p().and(name::name_with_dot_p().with_pos()))
        .map(|(_, opt_right)| opt_right.map(|(_, r)| r))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::types::*;

    #[test]
    fn test_for_loop() {
        let input = "FOR I = 1 TO 10\r\nFlint I\r\nNEXT";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            Statement::ForLoop(ForLoopNode {
                variable_name: Expression::var("I").at_rc(1, 5),
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
                variable_name: Expression::var("i").at_rc(1, 5),
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
                variable_name: Expression::var("I").at_rc(1, 5),
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
        let result = parse_file("FOR_NESTED.BAS").strip_location();
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::Print(PrintNode::one(
                    "Before the outer loop".as_lit_expr(1, 7)
                ))),
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: Expression::var("I").at_rc(2, 5),
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
                            variable_name: Expression::var("J").at_rc(4, 9),
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
                    variable_name: Expression::var("I").at_rc(2, 13),
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
        let result = parse_err_node(input);
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: STEP or end-of-statement"),
                Location::new(2, 23),
            )
        );
    }
}
