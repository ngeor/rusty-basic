use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::pc::loc::*;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

pub fn if_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QError>)> {
    map(
        if_first_demand_second(
            if_expr_then(),
            or(single_line_if_else(), multi_line_if()),
            || QError::SyntaxError("Expected single or multi line IF".to_string()),
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
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
    map(
        with_keyword_before(
            Keyword::If,
            with_some_whitespace_between(
                expression::expression_node(),
                demand_keyword(Keyword::Then),
                || QError::SyntaxError("Expected THEN".to_string()),
            ),
        ),
        |(l, _)| l,
    )
}

fn single_line_if_else<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> (
        EolReader<T>,
        Result<
            (
                StatementNodes,
                Vec<ConditionalBlockNode>,
                Option<StatementNodes>,
            ),
            QError,
        >,
    ),
> {
    map(
        if_first_maybe_second(
            single_line_if(),
            or(
                map(
                    crate::parser::pc::ws::with_leading(with_pos(comment::comment())),
                    |r| vec![r],
                ),
                single_line_else(),
            ),
        ),
        |(l, r)| (l, vec![], r),
    )
}

fn single_line_if<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QError>)> {
    statements::single_line_non_comment_statements()
}

fn single_line_else<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QError>)> {
    map(
        crate::parser::pc::ws::with_leading(and(
            try_read_keyword(Keyword::Else),
            statements::single_line_statements(),
        )),
        |(_, r)| r,
    )
}

fn multi_line_if<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> (
        EolReader<T>,
        Result<
            (
                StatementNodes,
                Vec<ConditionalBlockNode>,
                Option<StatementNodes>,
            ),
            QError,
        >,
    ),
> {
    map(
        if_first_demand_second(
            if_first_maybe_second(
                if_first_maybe_second(
                    statements::statements(read_keyword_if(|k| {
                        k == Keyword::End || k == Keyword::Else || k == Keyword::ElseIf
                    })),
                    else_if_blocks(),
                ),
                else_block(),
            ),
            end_if(),
            || QError::SyntaxError("Expected END IF".to_string()),
        ),
        |(((if_block, opt_else_if_blocks), opt_else), _)| {
            (if_block, opt_else_if_blocks.unwrap_or_default(), opt_else)
        },
    )
}

fn else_if_expr_then<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
    map(
        with_keyword_before(
            Keyword::ElseIf,
            with_some_whitespace_between(
                expression::expression_node(),
                demand_keyword(Keyword::Then),
                || QError::SyntaxError("Expected THEN".to_string()),
            ),
        ),
        |(l, _)| l,
    )
}

fn else_if_blocks<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Vec<ConditionalBlockNode>, QError>)> {
    take_zero_or_more(else_if_block(), |_| false)
}

fn else_if_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ConditionalBlockNode, QError>)> {
    map(
        if_first_demand_second(
            else_if_expr_then(),
            statements::statements(read_keyword_if(|k| {
                k == Keyword::End || k == Keyword::Else || k == Keyword::ElseIf
            })),
            || QError::SyntaxError("Expected statements after expression".to_string()),
        ),
        |(condition, statements)| ConditionalBlockNode {
            condition,
            statements,
        },
    )
}

fn else_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QError>)> {
    map(
        if_first_demand_second(
            try_read_keyword(Keyword::Else),
            // TODO add here an EOL else separator
            statements::statements(read_keyword_if(|k| k == Keyword::End)),
            || QError::SyntaxError("Expected statements after ELSE".to_string()),
        ),
        |(_, r)| r,
    )
}

fn end_if<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<(Keyword, String), QError>)> {
    with_keyword_before(Keyword::End, try_read_keyword(Keyword::If))
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
}
