use super::{
    unexpected, ConditionalBlockNode, ExpressionNode, IfBlockNode, Parser, ParserError, Statement,
    StatementNodes,
};
use crate::lexer::{Keyword, LexemeNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_if_block(&mut self) -> Result<Statement, ParserError> {
        self.read_demand_whitespace("Expected whitespace after IF keyword")?;
        let if_condition = self.read_demand_expression()?;
        self.read_demand_whitespace("Expected whitespace before THEN keyword")?;
        self.read_demand_keyword(Keyword::Then)?;
        let (opt_space, next) = self.read_preserve_whitespace()?;
        match next {
            // EOL, or whitespace and then EOL
            LexemeNode::EOL(_, _) => self.consume_if_block_multi_line(if_condition),
            _ => {
                match opt_space {
                    // whitespace and then something which wasn't EOL
                    Some(_) => self.consume_if_block_single_line(if_condition, next),
                    // THEN was not followed by space nor EOL
                    None => unexpected("Expected space or EOL", next),
                }
            }
        }
    }

    fn consume_if_block_single_line(
        &mut self,
        if_condition: ExpressionNode,
        next: LexemeNode,
    ) -> Result<Statement, ParserError> {
        let if_block = ConditionalBlockNode {
            condition: if_condition,
            statements: vec![self.demand_single_line_then_statement(next)?],
        };
        Ok(Statement::IfBlock(IfBlockNode {
            if_block: if_block,
            else_if_blocks: vec![],
            else_block: None,
        }))
    }

    fn consume_if_block_multi_line(
        &mut self,
        if_condition: ExpressionNode,
    ) -> Result<Statement, ParserError> {
        // read if statements
        let (if_statements, mut exit_lexeme) = self._demand_block_until_else_or_else_if_or_end()?;
        let if_block = ConditionalBlockNode {
            condition: if_condition,
            statements: if_statements,
        };
        // parse additional elseif blocks
        let mut else_if_blocks: Vec<ConditionalBlockNode> = vec![];
        while exit_lexeme.is_keyword(Keyword::ElseIf) {
            let (else_if_condition, else_if_statements, else_if_exit_lexeme) =
                self._demand_else_if_conditional_block()?;
            else_if_blocks.push(ConditionalBlockNode {
                condition: else_if_condition,
                statements: else_if_statements,
            });
            exit_lexeme = else_if_exit_lexeme;
        }
        // parse else block
        let else_block: Option<StatementNodes>;
        match exit_lexeme {
            LexemeNode::Keyword(Keyword::Else, _, _) => {
                else_block = self._demand_else_block().map(|x| Some(x))?;
            }
            LexemeNode::Keyword(Keyword::End, _, _) => {
                else_block = None;
            }
            _ => {
                return unexpected("Expected ELSE or END", exit_lexeme);
            }
        }
        // parse end if
        self.read_demand_whitespace("Expected whitespace after END keyword")?;
        self.read_demand_keyword(Keyword::If)?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;
        Ok(Statement::IfBlock(IfBlockNode {
            if_block: if_block,
            else_if_blocks: else_if_blocks,
            else_block: else_block,
        }))
    }

    fn _demand_else_if_conditional_block(
        &mut self,
    ) -> Result<(ExpressionNode, StatementNodes, LexemeNode), ParserError> {
        let condition = self._demand_else_if_condition()?;
        let (statements, next) = self._demand_block_until_else_or_else_if_or_end()?;
        Ok((condition, statements, next))
    }

    fn _demand_else_if_condition(&mut self) -> Result<ExpressionNode, ParserError> {
        self.read_demand_whitespace("Expected whitespace after ELSEIF")?;
        let condition = self.read_demand_expression()?;
        self.read_demand_whitespace("Expected whitespace before THEN keyword")?;
        self.read_demand_keyword(Keyword::Then)?;
        self.read_demand_eol_skipping_whitespace()?;
        Ok(condition)
    }

    fn _demand_block_until_else_or_else_if_or_end(
        &mut self,
    ) -> Result<(StatementNodes, LexemeNode), ParserError> {
        self.parse_statements(
            |x| match x {
                LexemeNode::Keyword(k, _, _) => {
                    *k == Keyword::Else || *k == Keyword::ElseIf || *k == Keyword::End
                }
                _ => false,
            },
            "Unexpected EOF while looking for end of ELSEIF",
        )
    }

    fn _demand_else_block(&mut self) -> Result<StatementNodes, ParserError> {
        self.read_demand_eol_skipping_whitespace()?;
        self.parse_statements(
            |x| x.is_keyword(Keyword::End),
            "Unexpected EOF while looking for end of ELSE block",
        )
        .map(|x| x.0)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{ConditionalBlockNode, IfBlockNode, Statement, TopLevelToken};

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
}
