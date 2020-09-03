use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

pub fn if_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        seq2(
            if_expr_then(),
            demand(
                or(single_line_if_else(), multi_line_if()),
                QError::syntax_error_fn("Expected: single or multi line IF"),
            ),
        ),
        |(condition, (statements, else_if_blocks, else_block))| {
            Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition,
                    statements,
                },
                else_if_blocks,
                else_block,
            })
        },
    )
}

// IF expr THEN ( single line if | multi line if)
// single line if   ::= <ws+>non-comment-statements-separated-by-colon ( single-line-else | comment-statement)
// single line else ::= ELSE non-comment-statements-separated-by-colon comment-statement
// multi line if    ::= statements else-if-blocks else-block END IF

fn if_expr_then<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    map(
        seq3(
            keyword(Keyword::If),
            expression::demand_back_guarded_expression_node(),
            demand_keyword(Keyword::Then),
        ),
        |(_, e, _)| e,
    )
}

fn single_line_if_else<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> ReaderResult<
        EolReader<T>,
        (
            StatementNodes,
            Vec<ConditionalBlockNode>,
            Option<StatementNodes>,
        ),
        QError,
    >,
> {
    map(
        opt_seq2(
            single_line_if(),
            or(
                map(
                    crate::parser::pc::ws::one_or_more_leading(with_pos(comment::comment())),
                    |r| vec![r],
                ),
                single_line_else(),
            ),
        ),
        |(l, r)| (l, vec![], r),
    )
}

fn single_line_if<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, StatementNodes, QError>> {
    statements::single_line_non_comment_statements()
}

fn single_line_else<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, StatementNodes, QError>> {
    map(
        crate::parser::pc::ws::one_or_more_leading(and(
            keyword(Keyword::Else),
            statements::single_line_statements(),
        )),
        |(_, r)| r,
    )
}

fn multi_line_if<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> ReaderResult<
        EolReader<T>,
        (
            StatementNodes,
            Vec<ConditionalBlockNode>,
            Option<StatementNodes>,
        ),
        QError,
    >,
> {
    map(
        seq2(
            opt_seq3(
                statements::statements(
                    or_vec(vec![
                        keyword(Keyword::End),
                        keyword(Keyword::Else),
                        keyword(Keyword::ElseIf),
                    ]),
                    QError::syntax_error_fn("Expected: end-of-statement"),
                ),
                else_if_blocks(),
                else_block(),
            ),
            demand(end_if(), QError::syntax_error_fn("Expected: END IF")),
        ),
        |((if_block, opt_else_if_blocks, opt_else), _)| {
            (if_block, opt_else_if_blocks.unwrap_or_default(), opt_else)
        },
    )
}

fn else_if_expr_then<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    map(
        seq3(
            keyword(Keyword::ElseIf),
            expression::demand_back_guarded_expression_node(),
            demand_keyword(Keyword::Then),
        ),
        |(_, e, _)| e,
    )
}

fn else_if_blocks<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Vec<ConditionalBlockNode>, QError>> {
    map_default_to_not_found(zero_or_more(map(else_if_block(), |x| (x, Some(())))))
}

fn else_if_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ConditionalBlockNode, QError>> {
    map(
        seq2(
            else_if_expr_then(),
            statements::statements(
                or_vec(vec![
                    keyword(Keyword::End),
                    keyword(Keyword::Else),
                    keyword(Keyword::ElseIf),
                ]),
                QError::syntax_error_fn("Expected: end-of-statement"),
            ),
        ),
        |(condition, statements)| ConditionalBlockNode {
            condition,
            statements,
        },
    )
}

fn else_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, StatementNodes, QError>> {
    drop_left(seq2(
        keyword(Keyword::Else),
        statements::statements(
            keyword(Keyword::End),
            QError::syntax_error_fn("Expected: end-of-statement"),
        ),
    ))
}

fn end_if<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (), QError>> {
    map(
        crate::parser::pc::ws::seq2(
            keyword(Keyword::End),
            demand(
                keyword(Keyword::If),
                QError::syntax_error_fn("Expected: IF after END"),
            ),
            QError::syntax_error_fn("Expected: whitespace after END"),
        ),
        |_| (),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{
        ConditionalBlockNode, Expression, IfBlockNode, Operand, Statement, TopLevelToken,
    };

    #[test]
    fn test_if() {
        let input = "IF X THEN\r\nPRINT X\r\nEND IF";
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["X".as_var_expr(2, 7)]
                    )
                    .at_rc(2, 1)]
                },
                else_if_blocks: vec![],
                else_block: None,
            }),
        );
    }

    #[test]
    fn test_if_single_line() {
        let input = "
        IF X THEN PRINT X
        SYSTEM
        ";
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::IfBlock(IfBlockNode {
                    if_block: ConditionalBlockNode {
                        condition: "X".as_var_expr(2, 12),
                        statements: vec![Statement::SubCall(
                            "PRINT".into(),
                            vec!["X".as_var_expr(2, 25)]
                        )
                        .at_rc(2, 19)]
                    },
                    else_if_blocks: vec![],
                    else_block: None
                })),
                TopLevelToken::Statement(Statement::SubCall("SYSTEM".into(), vec![]))
            ]
        );
    }

    #[test]
    fn test_if_else() {
        let input = r#"IF X THEN
    PRINT X
ELSE
    PRINT Y
END IF"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["X".as_var_expr(2, 11)]
                    )
                    .at_rc(2, 5)],
                },
                else_if_blocks: vec![],
                else_block: Some(vec![Statement::SubCall(
                    "PRINT".into(),
                    vec!["Y".as_var_expr(4, 11)]
                )
                .at_rc(4, 5)]),
            }),
        );
    }

    #[test]
    fn test_if_else_if() {
        let input = r#"IF X THEN
    PRINT X
ELSEIF Y THEN
    PRINT Y
END IF"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["X".as_var_expr(2, 11)]
                    )
                    .at_rc(2, 5)],
                },
                else_if_blocks: vec![ConditionalBlockNode {
                    condition: "Y".as_var_expr(3, 8),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["Y".as_var_expr(4, 11)]
                    )
                    .at_rc(4, 5)],
                }],
                else_block: None,
            }),
        );
    }

    #[test]
    fn test_if_else_if_two_branches() {
        let input = r#"IF X THEN
    PRINT X
ELSEIF Y THEN
    PRINT Y
ELSEIF Z THEN
    PRINT Z
END IF"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["X".as_var_expr(2, 11)]
                    )
                    .at_rc(2, 5)],
                },
                else_if_blocks: vec![
                    ConditionalBlockNode {
                        condition: "Y".as_var_expr(3, 8),
                        statements: vec![Statement::SubCall(
                            "PRINT".into(),
                            vec!["Y".as_var_expr(4, 11)]
                        )
                        .at_rc(4, 5)],
                    },
                    ConditionalBlockNode {
                        condition: "Z".as_var_expr(5, 8),
                        statements: vec![Statement::SubCall(
                            "PRINT".into(),
                            vec!["Z".as_var_expr(6, 11)]
                        )
                        .at_rc(6, 5)],
                    },
                ],
                else_block: None,
            }),
        );
    }

    #[test]
    fn test_if_else_if_else() {
        let input = r#"IF X THEN
    PRINT X
ELSEIF Y THEN
    PRINT Y
ELSE
    PRINT Z
END IF"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["X".as_var_expr(2, 11)]
                    )
                    .at_rc(2, 5)],
                },
                else_if_blocks: vec![ConditionalBlockNode {
                    condition: "Y".as_var_expr(3, 8),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["Y".as_var_expr(4, 11)]
                    )
                    .at_rc(4, 5)],
                }],
                else_block: Some(vec![Statement::SubCall(
                    "PRINT".into(),
                    vec!["Z".as_var_expr(6, 11)]
                )
                .at_rc(6, 5)]),
            })
        );
    }

    #[test]
    fn test_if_else_if_else_lower_case() {
        let input = r#"if x then
    print x
elseif y then
    print y
else
    print z
end if"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition: "x".as_var_expr(1, 4),
                    statements: vec![Statement::SubCall(
                        "print".into(),
                        vec!["x".as_var_expr(2, 11)]
                    )
                    .at_rc(2, 5)],
                },
                else_if_blocks: vec![ConditionalBlockNode {
                    condition: "y".as_var_expr(3, 8),
                    statements: vec![Statement::SubCall(
                        "print".into(),
                        vec!["y".as_var_expr(4, 11)]
                    )
                    .at_rc(4, 5)],
                }],
                else_block: Some(vec![Statement::SubCall(
                    "print".into(),
                    vec!["z".as_var_expr(6, 11)]
                )
                .at_rc(6, 5)]),
            })
        );
    }

    #[test]
    fn test_single_line_if_else() {
        let input = "IF ID = 0 THEN A$ = B$ ELSE A$ = C$";
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition: Expression::BinaryExpression(
                        Operand::Equal,
                        Box::new("ID".as_var_expr(1, 4)),
                        Box::new(0.as_lit_expr(1, 9))
                    )
                    .at_rc(1, 7),
                    statements: vec![
                        Statement::Assignment("A$".into(), "B$".as_var_expr(1, 21)).at_rc(1, 16)
                    ]
                },
                else_if_blocks: vec![],
                else_block: Some(vec![Statement::Assignment(
                    "A$".into(),
                    "C$".as_var_expr(1, 34)
                )
                .at_rc(1, 29)])
            })
        )
    }

    #[test]
    fn test_inline_comment() {
        let input = r#"
        IF A THEN     ' is a true?
            PRINT A   ' print a
        ELSEIF B THEN ' is b true?
            PRINT B   ' print b
        ELSE          ' nothing is true
            PRINT C   ' print c
        END IF        ' end if
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::IfBlock(IfBlockNode {
                    if_block: ConditionalBlockNode {
                        condition: "A".as_var_expr(2, 12),
                        statements: vec![
                            Statement::Comment(" is a true?".to_string()).at_rc(2, 23),
                            Statement::SubCall("PRINT".into(), vec!["A".as_var_expr(3, 19)])
                                .at_rc(3, 13),
                            Statement::Comment(" print a".to_string()).at_rc(3, 23)
                        ],
                    },
                    else_if_blocks: vec![ConditionalBlockNode {
                        condition: "B".as_var_expr(4, 16),
                        statements: vec![
                            Statement::Comment(" is b true?".to_string()).at_rc(4, 23),
                            Statement::SubCall("PRINT".into(), vec!["B".as_var_expr(5, 19)])
                                .at_rc(5, 13),
                            Statement::Comment(" print b".to_string()).at_rc(5, 23)
                        ],
                    }],
                    else_block: Some(vec![
                        Statement::Comment(" nothing is true".to_string()).at_rc(6, 23),
                        Statement::SubCall("PRINT".into(), vec!["C".as_var_expr(7, 19)])
                            .at_rc(7, 13),
                        Statement::Comment(" print c".to_string()).at_rc(7, 23)
                    ])
                }))
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" end if".to_string())).at_rc(8, 23)
            ]
        );
    }

    #[test]
    fn test_else_without_if() {
        let input = "ELSE";
        assert_eq!(parse_err(input), QError::ElseWithoutIf);
    }

    #[test]
    fn test_if_else_if_no_space_needed_if_condition_in_parenthesis() {
        let input = r#"
        IF(X>0)THEN
            PRINT "positive"
        ELSEIF(X<0)THEN
            PRINT "negative"
        ELSE
            PRINT "zero"
        END IF
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode {
                    condition: Expression::Parenthesis(Box::new(
                        Expression::BinaryExpression(
                            Operand::Greater,
                            Box::new("X".as_var_expr(2, 12)),
                            Box::new(0.as_lit_expr(2, 14)),
                        )
                        .at_rc(2, 13)
                    ))
                    .at_rc(2, 11),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["positive".as_lit_expr(3, 19)]
                    )
                    .at_rc(3, 13),],
                },
                else_if_blocks: vec![ConditionalBlockNode {
                    condition: Expression::Parenthesis(Box::new(
                        Expression::BinaryExpression(
                            Operand::Less,
                            Box::new("X".as_var_expr(4, 16)),
                            Box::new(0.as_lit_expr(4, 18)),
                        )
                        .at_rc(4, 17)
                    ))
                    .at_rc(4, 15),
                    statements: vec![Statement::SubCall(
                        "PRINT".into(),
                        vec!["negative".as_lit_expr(5, 19)]
                    )
                    .at_rc(5, 13),],
                }],
                else_block: Some(vec![Statement::SubCall(
                    "PRINT".into(),
                    vec!["zero".as_lit_expr(7, 19)]
                )
                .at_rc(7, 13),])
            }))
            .at_rc(2, 9),]
        );
    }
}
