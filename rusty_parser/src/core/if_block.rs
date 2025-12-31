use rusty_pc::*;

use crate::core::comment::comment_p;
use crate::core::expression::ws_expr_pos_ws_p;
use crate::core::statements::{
    ZeroOrMoreStatements, single_line_non_comment_statements_p, single_line_statements_p
};
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::{ParseError, *};

pub fn if_block_p() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq2(
        if_expr_then_p(),
        single_line_if_else_p()
            .or(multi_line_if_p())
            .or_syntax_error("Expected: single line or multi line IF"),
        |condition, (statements, else_if_blocks, else_block)| {
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
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

fn if_expr_then_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    seq3(
        keyword(Keyword::If),
        ws_expr_pos_ws_p().or_syntax_error("Expected: expression after IF"),
        keyword(Keyword::Then),
        |_, m, _| m,
    )
}

fn single_line_if_else_p() -> impl Parser<
    RcStringView,
    Output = (Statements, Vec<ConditionalBlock>, Option<Statements>),
    Error = ParseError,
> {
    single_line_non_comment_statements_p().and_opt(
        // comment or ELSE
        whitespace()
            .and(comment_p().with_pos(), |_, s| vec![s])
            .or(single_line_else_p()),
        |l, r| (l, vec![], r),
    )
}

fn single_line_else_p() -> impl Parser<RcStringView, Output = Statements, Error = ParseError> {
    whitespace()
        .and_tuple(keyword(Keyword::Else))
        .and_keep_right(
            single_line_statements_p().or_syntax_error("Expected statements for single line ELSE"),
        )
}

fn multi_line_if_p() -> impl Parser<
    RcStringView,
    Output = (Statements, Vec<ConditionalBlock>, Option<Statements>),
    Error = ParseError,
> {
    seq4(
        ZeroOrMoreStatements::new_multi(vec![Keyword::End, Keyword::Else, Keyword::ElseIf]),
        else_if_block_p().zero_or_more(),
        else_block_p().to_option(),
        keyword_pair(Keyword::End, Keyword::If),
        |if_block, else_if_blocks, opt_else, _| (if_block, else_if_blocks, opt_else),
    )
}

fn else_if_expr_then_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    seq3(
        keyword(Keyword::ElseIf),
        ws_expr_pos_ws_p().or_syntax_error("Expected: expression after ELSEIF"),
        keyword(Keyword::Then),
        |_, m, _| m,
    )
}

fn else_if_block_p() -> impl Parser<RcStringView, Output = ConditionalBlock, Error = ParseError> {
    seq2(
        else_if_expr_then_p(),
        ZeroOrMoreStatements::new_multi(vec![Keyword::End, Keyword::Else, Keyword::ElseIf]),
        |condition, statements| ConditionalBlock {
            condition,
            statements,
        },
    )
}

fn else_block_p() -> impl Parser<RcStringView, Output = Statements, Error = ParseError> {
    keyword(Keyword::Else).and_keep_right(ZeroOrMoreStatements::new(Keyword::End))
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::error::ParseError;
    use crate::test_utils::*;
    use crate::{assert_parser_err, *};
    #[test]
    fn test_if() {
        let input = "IF X THEN\r\nFlint X\r\nEND IF";
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![
                        Statement::sub_call("Flint".into(), vec!["X".as_var_expr(2, 7)])
                            .at_rc(2, 1)
                    ]
                },
                else_if_blocks: vec![],
                else_block: None,
            }),
        );
    }

    #[test]
    fn test_if_single_line() {
        let input = "
        IF X THEN Flint X
        SYSTEM
        ";
        let program = parse_str_no_pos(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::IfBlock(IfBlock {
                    if_block: ConditionalBlock {
                        condition: "X".as_var_expr(2, 12),
                        statements: vec![
                            Statement::sub_call("Flint".into(), vec!["X".as_var_expr(2, 25)])
                                .at_rc(2, 19)
                        ]
                    },
                    else_if_blocks: vec![],
                    else_block: None
                })),
                GlobalStatement::Statement(Statement::System)
            ]
        );
    }

    #[test]
    fn test_if_else() {
        let input = r#"IF X THEN
    Flint X
ELSE
    Flint Y
END IF"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![
                        Statement::sub_call("Flint".into(), vec!["X".as_var_expr(2, 11)])
                            .at_rc(2, 5)
                    ],
                },
                else_if_blocks: vec![],
                else_block: Some(vec![
                    Statement::sub_call("Flint".into(), vec!["Y".as_var_expr(4, 11)]).at_rc(4, 5)
                ]),
            }),
        );
    }

    #[test]
    fn test_if_else_if() {
        let input = r#"IF X THEN
    Flint X
ELSEIF Y THEN
    Flint Y
END IF"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![
                        Statement::sub_call("Flint".into(), vec!["X".as_var_expr(2, 11)])
                            .at_rc(2, 5)
                    ],
                },
                else_if_blocks: vec![ConditionalBlock {
                    condition: "Y".as_var_expr(3, 8),
                    statements: vec![
                        Statement::sub_call("Flint".into(), vec!["Y".as_var_expr(4, 11)])
                            .at_rc(4, 5)
                    ],
                }],
                else_block: None,
            }),
        );
    }

    #[test]
    fn test_if_else_if_two_branches() {
        let input = r#"IF X THEN
    Flint X
ELSEIF Y THEN
    Flint Y
ELSEIF Z THEN
    Flint Z
END IF"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![
                        Statement::sub_call("Flint".into(), vec!["X".as_var_expr(2, 11)])
                            .at_rc(2, 5)
                    ],
                },
                else_if_blocks: vec![
                    ConditionalBlock {
                        condition: "Y".as_var_expr(3, 8),
                        statements: vec![
                            Statement::sub_call("Flint".into(), vec!["Y".as_var_expr(4, 11)])
                                .at_rc(4, 5)
                        ],
                    },
                    ConditionalBlock {
                        condition: "Z".as_var_expr(5, 8),
                        statements: vec![
                            Statement::sub_call("Flint".into(), vec!["Z".as_var_expr(6, 11)])
                                .at_rc(6, 5)
                        ],
                    },
                ],
                else_block: None,
            }),
        );
    }

    #[test]
    fn test_if_else_if_else() {
        let input = r#"IF X THEN
    Flint X
ELSEIF Y THEN
    Flint Y
ELSE
    Flint Z
END IF"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
                    condition: "X".as_var_expr(1, 4),
                    statements: vec![
                        Statement::sub_call("Flint".into(), vec!["X".as_var_expr(2, 11)])
                            .at_rc(2, 5)
                    ],
                },
                else_if_blocks: vec![ConditionalBlock {
                    condition: "Y".as_var_expr(3, 8),
                    statements: vec![
                        Statement::sub_call("Flint".into(), vec!["Y".as_var_expr(4, 11)])
                            .at_rc(4, 5)
                    ],
                }],
                else_block: Some(vec![
                    Statement::sub_call("Flint".into(), vec!["Z".as_var_expr(6, 11)]).at_rc(6, 5)
                ]),
            })
        );
    }

    #[test]
    fn test_if_else_if_else_lower_case() {
        let input = r#"if x then
    flint x
elseif y then
    flint y
else
    flint z
end if"#;
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
                    condition: "x".as_var_expr(1, 4),
                    statements: vec![
                        Statement::sub_call("flint".into(), vec!["x".as_var_expr(2, 11)])
                            .at_rc(2, 5)
                    ],
                },
                else_if_blocks: vec![ConditionalBlock {
                    condition: "y".as_var_expr(3, 8),
                    statements: vec![
                        Statement::sub_call("flint".into(), vec!["y".as_var_expr(4, 11)])
                            .at_rc(4, 5)
                    ],
                }],
                else_block: Some(vec![
                    Statement::sub_call("flint".into(), vec!["z".as_var_expr(6, 11)]).at_rc(6, 5)
                ]),
            })
        );
    }

    #[test]
    fn test_single_line_if_else() {
        let input = "IF ID = 0 THEN A$ = B$ ELSE A$ = C$";
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
                    condition: Expression::BinaryExpression(
                        Operator::Equal,
                        Box::new("ID".as_var_expr(1, 4)),
                        Box::new(0.as_lit_expr(1, 9)),
                        ExpressionType::Unresolved
                    )
                    .at_rc(1, 7),
                    statements: vec![
                        Statement::assignment(
                            Expression::var_unresolved("A$"),
                            "B$".as_var_expr(1, 21)
                        )
                        .at_rc(1, 16)
                    ]
                },
                else_if_blocks: vec![],
                else_block: Some(vec![
                    Statement::assignment(
                        Expression::var_unresolved("A$"),
                        "C$".as_var_expr(1, 34)
                    )
                    .at_rc(1, 29)
                ])
            })
        )
    }

    #[test]
    fn test_inline_comment() {
        let input = r#"
        IF A THEN     ' is a true?
            Flint A   ' print a
        ELSEIF B THEN ' is b true?
            Flint B   ' print b
        ELSE          ' nothing is true
            Flint C   ' print c
        END IF        ' end if
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::IfBlock(IfBlock {
                    if_block: ConditionalBlock {
                        condition: "A".as_var_expr(2, 12),
                        statements: vec![
                            Statement::Comment(" is a true?".to_string()).at_rc(2, 23),
                            Statement::sub_call("Flint".into(), vec!["A".as_var_expr(3, 19)])
                                .at_rc(3, 13),
                            Statement::Comment(" print a".to_string()).at_rc(3, 23)
                        ],
                    },
                    else_if_blocks: vec![ConditionalBlock {
                        condition: "B".as_var_expr(4, 16),
                        statements: vec![
                            Statement::Comment(" is b true?".to_string()).at_rc(4, 23),
                            Statement::sub_call("Flint".into(), vec!["B".as_var_expr(5, 19)])
                                .at_rc(5, 13),
                            Statement::Comment(" print b".to_string()).at_rc(5, 23)
                        ],
                    }],
                    else_block: Some(vec![
                        Statement::Comment(" nothing is true".to_string()).at_rc(6, 23),
                        Statement::sub_call("Flint".into(), vec!["C".as_var_expr(7, 19)])
                            .at_rc(7, 13),
                        Statement::Comment(" print c".to_string()).at_rc(7, 23)
                    ])
                }))
                .at_rc(2, 9),
                GlobalStatement::Statement(Statement::Comment(" end if".to_string())).at_rc(8, 23)
            ]
        );
    }

    #[test]
    fn test_else_without_if() {
        let input = "ELSE";
        assert_parser_err!(input, ParseError::ElseWithoutIf);
    }

    #[test]
    fn test_if_else_if_no_space_needed_if_condition_in_parenthesis() {
        let input = r#"
        IF(X>0)THEN
            Flint "positive"
        ELSEIF(X<0)THEN
            Flint "negative"
        ELSE
            Flint "zero"
        END IF
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::IfBlock(IfBlock {
                    if_block: ConditionalBlock {
                        condition: Expression::Parenthesis(Box::new(
                            Expression::BinaryExpression(
                                Operator::Greater,
                                Box::new("X".as_var_expr(2, 12)),
                                Box::new(0.as_lit_expr(2, 14)),
                                ExpressionType::Unresolved
                            )
                            .at_rc(2, 13)
                        ))
                        .at_rc(2, 11),
                        statements: vec![
                            Statement::sub_call(
                                "Flint".into(),
                                vec!["positive".as_lit_expr(3, 19)]
                            )
                            .at_rc(3, 13),
                        ],
                    },
                    else_if_blocks: vec![ConditionalBlock {
                        condition: Expression::Parenthesis(Box::new(
                            Expression::BinaryExpression(
                                Operator::Less,
                                Box::new("X".as_var_expr(4, 16)),
                                Box::new(0.as_lit_expr(4, 18)),
                                ExpressionType::Unresolved
                            )
                            .at_rc(4, 17)
                        ))
                        .at_rc(4, 15),
                        statements: vec![
                            Statement::sub_call(
                                "Flint".into(),
                                vec!["negative".as_lit_expr(5, 19)]
                            )
                            .at_rc(5, 13),
                        ],
                    }],
                    else_block: Some(vec![
                        Statement::sub_call("Flint".into(), vec!["zero".as_lit_expr(7, 19)])
                            .at_rc(7, 13),
                    ])
                }))
                .at_rc(2, 9),
            ]
        );
    }

    #[test]
    fn test_if_expr_left_parenthesis() {
        let input = r#"
        IF (A + B) >= C THEN
            BEEP
        END IF
        "#;
        let statement = parse(input).demand_single_statement();
        assert_eq!(
            statement,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock {
                    condition: Expression::BinaryExpression(
                        Operator::GreaterOrEqual,
                        Box::new(
                            Expression::Parenthesis(Box::new(
                                Expression::BinaryExpression(
                                    Operator::Plus,
                                    Box::new("A".as_var_expr(2, 13)),
                                    Box::new("B".as_var_expr(2, 17)),
                                    ExpressionType::Unresolved
                                )
                                .at_rc(2, 15)
                            ))
                            .at_rc(2, 12)
                        ),
                        Box::new("C".as_var_expr(2, 23)),
                        ExpressionType::Unresolved
                    )
                    .at_rc(2, 20),
                    statements: vec![Statement::sub_call("BEEP".into(), vec![]).at_rc(3, 13)]
                },
                else_if_blocks: vec![],
                else_block: None
            })
        );
    }
}
