use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::pc::combine::combine_some;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

// FOR I = 0 TO 5 STEP 1
// statements
// NEXT (I)

pub fn for_loop<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        seq3(
            combine_some(parse_for(), parse_step),
            statements::statements(
                try_read_keyword(Keyword::Next),
                QError::syntax_error_fn("Expected: STEP or end-of-statement"),
            ),
            demand(next_counter(), || QError::ForWithoutNext),
        ),
        |(((variable_name, lower_bound, upper_bound), opt_step), statements, next_counter)| {
            Statement::ForLoop(ForLoopNode {
                variable_name,
                lower_bound,
                upper_bound,
                step: opt_step,
                statements,
                next_counter,
            })
        },
    )
}

fn parse_for<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> ReaderResult<EolReader<T>, (NameNode, ExpressionNode, ExpressionNode), QError>,
> {
    map(
        seq7(
            try_read_keyword(Keyword::For),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace after FOR"),
            ),
            demand(
                name::name_node(),
                QError::syntax_error_fn("Expected: name after FOR"),
            ),
            demand(
                crate::parser::pc::ws::zero_or_more_leading(try_read('=')),
                QError::syntax_error_fn("Expected: = after name"),
            ),
            expression::demand_back_guarded_expression_node(),
            demand(
                try_read_keyword(Keyword::To),
                QError::syntax_error_fn("Expected: TO"),
            ),
            expression::demand_guarded_expression_node(),
        ),
        |(_, _, n, _, l, _, u)| (n, l, u),
    )
}

fn parse_step<T: BufRead + 'static>(
    for_expr: &(NameNode, ExpressionNode, ExpressionNode),
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    let (_, _, upper) = for_expr;
    let parenthesis = upper.is_parenthesis();
    if parenthesis {
        drop_left(seq2(
            crate::parser::pc::ws::zero_or_more_leading(try_read_keyword(Keyword::Step)),
            expression::demand_guarded_expression_node(),
        ))
    } else {
        drop_left(seq2(
            crate::parser::pc::ws::one_or_more_leading(try_read_keyword(Keyword::Step)),
            expression::demand_guarded_expression_node(),
        ))
    }
}

fn next_counter<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Option<NameNode>, QError>> {
    map(
        opt_seq2(
            try_read_keyword(Keyword::Next),
            crate::parser::pc::ws::one_or_more_leading(name::name_node()),
        ),
        |(_, opt_second)| opt_second,
    )
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

    #[test]
    fn test_no_space_before_step() {
        let input = "
        FOR I = 0 TO 2STEP 1
        NEXT I
        ";
        let result = parse_main_str(input).unwrap_err();
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: STEP or end-of-statement"),
                Location::new(2, 23)
            )
        );
    }
}
