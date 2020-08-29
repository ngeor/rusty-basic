use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

// SELECT CASE expr ' comment
// CASE 1
// CASE IS >= 2
// CASE 5 TO 7
// CASE ELSE
// END SELECT

pub fn select_case<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QError>)> {
    map(
        seq2(
            opt_seq4(
                parse_select_case_expr(),
                // parse inline comments after SELECT
                statements::statements(read_keyword_if(|k| {
                    k == Keyword::Case || k == Keyword::End
                })),
                case_blocks(),
                case_else_statements(),
            ),
            demand(
                parse_end_select(),
                QError::syntax_error_fn("Expected END SELECT"),
            ),
        ),
        |((expr, inline_statements, opt_blocks, opt_else), _)| {
            Statement::SelectCase(SelectCaseNode {
                expr,
                case_blocks: opt_blocks.unwrap_or_default(),
                else_block: opt_else,
                inline_comments: inline_statements
                    .unwrap_or_default()
                    .into_iter()
                    .map(|x| match x {
                        Locatable {
                            element: Statement::Comment(text),
                            pos,
                        } => Locatable::new(text, pos),
                        _ => panic!("only comments are allowed - todo improve this"),
                    })
                    .collect(),
            })
        },
    )
}

fn parse_select_case_expr<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
    map(
        crate::parser::pc::ws::seq3(
            try_read_keyword(Keyword::Select),
            demand(
                try_read_keyword(Keyword::Case),
                QError::syntax_error_fn("Expected CASE"),
            ),
            demand(
                expression::expression_node(),
                QError::syntax_error_fn("Expected expression"),
            ),
            QError::syntax_error_fn_fn("Expected whitespace"),
        ),
        |(_, _, e)| e,
    )
}

fn case_blocks<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Vec<CaseBlockNode>, QError>)> {
    map_default_to_not_found(zero_or_more(map(case_block(), |x| (x, Some(())))))
}

fn case_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<CaseBlockNode, QError>)> {
    map(
        seq2(
            case_expr(),
            demand(
                statements::statements(read_keyword_if(|k| {
                    k == Keyword::Case || k == Keyword::End
                })),
                QError::syntax_error_fn("Expected statements after case expression"),
            ),
        ),
        |(expr, statements)| CaseBlockNode { expr, statements },
    )
}

fn case_expr<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<CaseExpression, QError>)> {
    drop_left(and(
        try_read_keyword(Keyword::Case),
        crate::parser::pc::ws::one_or_more_leading(drop_left(and(
            negate(try_read_keyword(Keyword::Else)),
            or(case_expr_is(), case_expr_to_or_simple()),
        ))),
    ))
}

fn case_expr_is<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<CaseExpression, QError>)> {
    map(
        seq4(
            try_read_keyword(Keyword::Is),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected whitespace after IS"),
            ),
            demand(
                expression::operand(false),
                QError::syntax_error_fn("Expected operand after IS"),
            ),
            demand(
                crate::parser::pc::ws::zero_or_more_leading(expression::expression_node()),
                QError::syntax_error_fn("Expected expression"),
            ),
        ),
        |(_, _, op, r)| CaseExpression::Is(op.strip_location(), r),
    )
}

fn case_expr_to_or_simple<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<CaseExpression, QError>)> {
    map(
        opt_seq2(
            expression::expression_node(),
            drop_left(and(
                crate::parser::pc::ws::one_or_more(),
                drop_left(crate::parser::pc::ws::seq2(
                    try_read_keyword(Keyword::To),
                    demand(
                        expression::expression_node(),
                        QError::syntax_error_fn("Expected expression after TO"),
                    ),
                    QError::syntax_error_fn("Expected whitespace after TO"),
                )),
            )),
        ),
        |(l, opt_r)| match opt_r {
            Some(r) => CaseExpression::Range(l, r),
            None => CaseExpression::Simple(l),
        },
    )
}

fn case_else_statements<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QError>)> {
    // all other CASE variations have been parsed by the time we're here, it must be CASE ELSE
    map(
        seq4(
            try_read_keyword(Keyword::Case),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected whitespace after CASE"),
            ),
            demand(
                try_read_keyword(Keyword::Else),
                QError::syntax_error_fn("Expected ELSE after CASE"),
            ),
            statements::statements(try_read_keyword(Keyword::End)),
        ),
        |(_, _, _, r)| r,
    )
}

fn parse_end_select<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<(), QError>)> {
    map(
        crate::parser::pc::ws::seq2(
            try_read_keyword(Keyword::End),
            demand(
                try_read_keyword(Keyword::Select),
                QError::syntax_error_fn("Expected SELECT"),
            ),
            QError::syntax_error_fn("Expected whitespace after CASE"),
        ),
        |(_, _)| (),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::*;

    #[test]
    fn test_inline_comment() {
        let input = r#"
        SELECT CASE X ' testing for x
        CASE 1        ' is it one?
        PRINT "One"   ' print it
        CASE ELSE     ' something else?
        PRINT "Nope"  ' print nope
        END SELECT    ' end of select
        "#;
        let result = parse(input);
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::SelectCase(SelectCaseNode {
                    expr: "X".as_var_expr(2, 21),
                    inline_comments: vec![" testing for x".to_string().at_rc(2, 23)],
                    case_blocks: vec![CaseBlockNode {
                        expr: CaseExpression::Simple(1.as_lit_expr(3, 14)),
                        statements: vec![
                            Statement::Comment(" is it one?".to_string()).at_rc(3, 23),
                            Statement::SubCall("PRINT".into(), vec!["One".as_lit_expr(4, 15)])
                                .at_rc(4, 9),
                            Statement::Comment(" print it".to_string()).at_rc(4, 23),
                        ]
                    }],
                    else_block: Some(vec![
                        Statement::Comment(" something else?".to_string()).at_rc(5, 23),
                        Statement::SubCall("PRINT".into(), vec!["Nope".as_lit_expr(6, 15)])
                            .at_rc(6, 9),
                        Statement::Comment(" print nope".to_string()).at_rc(6, 23),
                    ]),
                }))
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" end of select".to_string()))
                    .at_rc(7, 23)
            ]
        );
    }

    #[test]
    fn test_no_case() {
        let input = r#"
        SELECT CASE X
        END SELECT
        "#;
        let result = parse(input);
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::SelectCase(SelectCaseNode {
                    expr: "X".as_var_expr(2, 21),
                    inline_comments: vec![],
                    case_blocks: vec![],
                    else_block: None
                }))
                .at_rc(2, 9)
            ]
        );
    }

    #[test]
    fn test_inline_comment_next_line() {
        let input = r#"
        SELECT CASE X ' testing for x
        ' first case
        CASE 1        ' is it one?
        PRINT "One"   ' print it
        END SELECT
        "#;
        let result = parse(input);
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::SelectCase(SelectCaseNode {
                    expr: "X".as_var_expr(2, 21),
                    inline_comments: vec![
                        " testing for x".to_string().at_rc(2, 23),
                        " first case".to_string().at_rc(3, 9)
                    ],
                    case_blocks: vec![CaseBlockNode {
                        expr: CaseExpression::Simple(1.as_lit_expr(4, 14)),
                        statements: vec![
                            Statement::Comment(" is it one?".to_string()).at_rc(4, 23),
                            Statement::SubCall("PRINT".into(), vec!["One".as_lit_expr(5, 15)])
                                .at_rc(5, 9),
                            Statement::Comment(" print it".to_string()).at_rc(5, 23),
                        ]
                    }],
                    else_block: None
                }))
                .at_rc(2, 9)
            ]
        );
    }
}
