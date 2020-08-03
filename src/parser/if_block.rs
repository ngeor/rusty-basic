use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::error::*;
use crate::parser::expression;
use crate::parser::statements::*;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    if !lexer.peek()?.is_keyword(Keyword::If) {
        return Ok(None);
    }

    let pos = lexer.read()?.location();
    read_demand_whitespace(lexer, "Expected whitespace after IF keyword")?;
    let if_condition = demand(lexer, expression::try_read, "Expected expression after IF")?;
    read_demand_whitespace(lexer, "Expected whitespace before THEN keyword")?;
    read_demand_keyword(lexer, Keyword::Then)?;
    let is_multi_line = is_multi_line(lexer)?;
    let if_block = read_if_block(lexer, if_condition, is_multi_line)?;
    let mut else_if_blocks: Vec<ConditionalBlockNode> = vec![];
    loop {
        let else_if_block = try_read_else_if_block(lexer)?;
        match else_if_block {
            Some(e) => else_if_blocks.push(e),
            None => break,
        }
    }
    let else_block = try_read_else_block(lexer, is_multi_line)?;
    if is_multi_line {
        read_demand_keyword(lexer, Keyword::End)?;
        read_demand_whitespace(lexer, "Expected space after END")?;
        read_demand_keyword(lexer, Keyword::If)?;
    }

    Ok(Some(
        Statement::IfBlock(IfBlockNode {
            if_block,
            else_if_blocks,
            else_block,
        })
        .at(pos),
    ))
}

/// Read the IF block, up to the first ELSE IF or ELSE or END IF
fn read_if_block<T: BufRead>(
    lexer: &mut BufLexer<T>,
    condition: ExpressionNode,
    is_multi_line: bool,
) -> Result<ConditionalBlockNode, ParserError> {
    let statements = if is_multi_line {
        parse_statements(lexer, exit_predicate_if_multi_line, "Unterminated IF")?
    } else {
        parse_statements_with_options(
            lexer,
            exit_predicate_if_single_line,
            "Unterminated IF",
            ParseStatementsOptions {
                first_statement_separated_by_whitespace: true,
            },
        )?
    };
    Ok(ConditionalBlockNode {
        condition,
        statements,
    })
}

fn try_read_else_if_block<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<ConditionalBlockNode>, ParserError> {
    if !lexer.peek()?.is_keyword(Keyword::ElseIf) {
        return Ok(None);
    }
    lexer.read()?;
    read_demand_whitespace(lexer, "Expected whitespace after ELSEIF keyword")?;
    let condition = demand(
        lexer,
        expression::try_read,
        "Expected expression out of ELISEIF",
    )?;
    read_demand_whitespace(lexer, "Expected whitespace before THEN keyword")?;
    read_demand_keyword(lexer, Keyword::Then)?;
    let statements = parse_statements(lexer, exit_predicate_if_multi_line, "Unterminated IF")?;
    Ok(Some(ConditionalBlockNode {
        condition,
        statements,
    }))
}

fn try_read_else_block<T: BufRead>(
    lexer: &mut BufLexer<T>,
    is_multi_line: bool,
) -> Result<Option<StatementNodes>, ParserError> {
    if !lexer.peek()?.is_keyword(Keyword::Else) {
        return Ok(None);
    }
    lexer.read()?;
    if is_multi_line {
        parse_statements(lexer, exit_predicate_else_multi_line, "Unterminated ELSE")
            .map(|x| Some(x))
    } else {
        parse_statements_with_options(
            lexer,
            exit_predicate_else_single_line,
            "Unterminated ELSE",
            ParseStatementsOptions {
                first_statement_separated_by_whitespace: true,
            },
        )
        .map(|x| Some(x))
    }
}

fn is_multi_line<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<bool, ParserError> {
    // if we find EOL or comment, it's multi-line
    lexer.begin_transaction();
    skip_whitespace(lexer)?;
    let p = lexer.peek()?;
    let is_multi_line = p.is_eol() || p.is_symbol('\'');
    lexer.rollback_transaction()?;
    Ok(is_multi_line)
}

fn exit_predicate_if_single_line(l: LexemeNode) -> bool {
    l.is_eof() || l.is_eol() || l.is_keyword(Keyword::ElseIf) || l.is_keyword(Keyword::Else)
}

fn exit_predicate_if_multi_line(l: LexemeNode) -> bool {
    l.is_keyword(Keyword::ElseIf) || l.is_keyword(Keyword::Else) || l.is_keyword(Keyword::End)
}

fn exit_predicate_else_single_line(l: LexemeNode) -> bool {
    l.is_eol_or_eof()
}

fn exit_predicate_else_multi_line(l: LexemeNode) -> bool {
    l.is_keyword(Keyword::End)
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
}
