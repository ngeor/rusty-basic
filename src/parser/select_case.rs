use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

// SELECT CASE expr ' comment
// CASE 1
// CASE IS >= 2
// CASE 5 TO 7
// CASE ELSE
// END SELECT

// CASE <ws+> ELSE (priority)
// CASE <expr> TO <expr>
// CASE <ws+> IS <operand> <expr>
// CASE <expr>

pub fn select_case<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        seq2(
            seq3(
                parse_select_case_expr(),
                // parse inline comments after SELECT
                comment::comments(),
                zero_or_more(parse_case_any()),
            ),
            demand(
                parse_end_select(),
                QError::syntax_error_fn("Expected: END SELECT"),
            ),
        ),
        |((expr, inline_comments, v), _)| {
            let mut case_blocks: Vec<CaseBlockNode> = vec![];
            let mut else_block: Option<StatementNodes> = None;
            for (opt_case_expr, s) in v {
                match opt_case_expr {
                    Some(case_expr) => {
                        case_blocks.push(CaseBlockNode {
                            expr: case_expr,
                            statements: s,
                        });
                    }
                    None => {
                        if else_block.is_some() {
                            panic!("Multiple case else blocks");
                        }
                        else_block = Some(s);
                    }
                }
            }
            Statement::SelectCase(SelectCaseNode {
                expr,
                case_blocks,
                else_block,
                inline_comments,
            })
        },
    )
}

fn parse_select_case_expr<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    map(
        seq3(
            try_read_keyword(Keyword::Select),
            demand_guarded_keyword(Keyword::Case),
            expression::demand_guarded_expression_node(),
        ),
        |(_, _, e)| e,
    )
}

enum ExprOrElse {
    Expr(CaseExpression),
    Else,
}

fn parse_case_any<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> ReaderResult<
        EolReader<T>,
        ((Option<CaseExpression>, StatementNodes), Option<()>),
        QError,
    >,
> {
    map(
        seq2(
            try_read_keyword(Keyword::Case),
            or_vec(vec![
                seq2(
                    parse_case_else(),
                    statements::statements(
                        try_read_keyword(Keyword::End),
                        QError::syntax_error_fn("Expected: end-of-statement"),
                    ),
                ),
                seq2(
                    parse_case_is(),
                    statements::statements(
                        read_keyword_if(|k| k == Keyword::Case || k == Keyword::End),
                        QError::syntax_error_fn("Expected: end-of-statement"),
                    ),
                ),
                seq2(
                    parse_case_simple_or_range(),
                    statements::statements(
                        read_keyword_if(|k| k == Keyword::Case || k == Keyword::End),
                        QError::syntax_error_fn("Expected: TO or end-of-statement"),
                    ),
                ),
                // TODO make pc for this
                Box::new(|reader| {
                    Err((
                        reader,
                        QError::syntax_error("Expected: ELSE, IS, expression"),
                    ))
                }),
            ]),
        ),
        |(_, (expr_or_else, s))| match expr_or_else {
            ExprOrElse::Expr(e) => ((Some(e), s), Some(())),
            _ => ((None, s), None),
        },
    )
}

fn parse_case_else<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExprOrElse, QError>> {
    map(
        and(
            crate::parser::pc::ws::one_or_more(),
            try_read_keyword(Keyword::Else),
        ),
        |_| ExprOrElse::Else,
    )
}

fn parse_case_is<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExprOrElse, QError>> {
    map(
        seq5(
            and(
                crate::parser::pc::ws::one_or_more(),
                try_read_keyword(Keyword::Is),
            ),
            crate::parser::pc::ws::zero_or_more(),
            demand(
                // TODO demand only relational operator
                expression::operand(false),
                QError::syntax_error_fn("Expected: operand after IS"),
            ),
            crate::parser::pc::ws::zero_or_more(),
            expression::demand_expression_node(),
        ),
        |(_, _, op, _, r)| ExprOrElse::Expr(CaseExpression::Is(op.strip_location(), r)),
    )
}

fn parse_case_simple_or_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExprOrElse, QError>> {
    map(
        opt_seq2_comb(expression::guarded_expression_node(), parse_range()),
        |(l, opt_r)| match opt_r {
            Some(r) => ExprOrElse::Expr(CaseExpression::Range(l, r)),
            _ => ExprOrElse::Expr(CaseExpression::Simple(l)),
        },
    )
}

fn parse_range<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>, &ExpressionNode) -> ReaderResult<EolReader<T>, ExpressionNode, QError>>
{
    Box::new(move |reader, first_expr_ref| {
        // TODO refactor this
        let parenthesis = first_expr_ref.is_parenthesis();
        if parenthesis {
            drop_left(drop_left(and(
                crate::parser::pc::ws::zero_or_more(),
                seq2(
                    try_read_keyword(Keyword::To),
                    demand(
                        expression::guarded_expression_node(),
                        QError::syntax_error_fn("Expected: expression after TO"),
                    ),
                ),
            )))(reader)
        } else {
            // one or more
            drop_left(drop_left(and(
                crate::parser::pc::ws::one_or_more(),
                seq2(
                    try_read_keyword(Keyword::To),
                    demand(
                        expression::guarded_expression_node(),
                        QError::syntax_error_fn("Expected: expression after TO"),
                    ),
                ),
            )))(reader)
        }
    })
}

fn parse_end_select<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (), QError>> {
    map(
        seq2(
            try_read_keyword(Keyword::End),
            demand_guarded_keyword(Keyword::Select),
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
    fn test_select_case_inline_comment() {
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

    #[test]
    fn test_no_space_after_select_case() {
        let input = "
        SELECT CASE1
        END SELECT";
        let result = parse_main_str(input).unwrap_err();
        assert_eq!(
            result,
            QErrorNode::Pos(QError::syntax_error("Expected: CASE"), Location::new(2, 16))
        );
    }

    #[test]
    fn test_no_space_after_case() {
        let input = "
        SELECT CASE X
        CASE1
        END SELECT";
        let result = parse_main_str(input).unwrap_err();
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: END SELECT"),
                Location::new(3, 9)
            )
        );
    }

    #[test]
    fn test_no_space_unfinished_to() {
        let input = "
        SELECT CASE X
        CASE 1 TO
        END SELECT";
        let result = parse_main_str(input).unwrap_err();
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: expression after TO"),
                Location::new(3, 18)
            )
        );
    }

    #[test]
    fn test_no_space_before_to_unfinished_to() {
        let input = "
        SELECT CASE X
        CASE 1TO
        END SELECT";
        let result = parse_main_str(input).unwrap_err();
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: TO or end-of-statement"),
                Location::new(3, 15)
            )
        );
    }

    #[test]
    fn test_no_space_around_to() {
        let input = "
        SELECT CASE X
        CASE 1TO2
        END SELECT";
        let result = parse_main_str(input).unwrap_err();
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: TO or end-of-statement"),
                Location::new(3, 15)
            )
        );
    }

    #[test]
    fn test_no_space_after_to() {
        let input = "
        SELECT CASE X
        CASE 1 TO2
        END SELECT";
        let result = parse_main_str(input).unwrap_err();
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: TO or end-of-statement"),
                Location::new(3, 16)
            )
        );
    }

    #[test]
    fn test_no_space_before_to() {
        let input = "
        SELECT CASE X
        CASE 1TO 2
        END SELECT";
        let result = parse_main_str(input).unwrap_err();
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: TO or end-of-statement"),
                Location::new(3, 15)
            )
        );
    }
}
