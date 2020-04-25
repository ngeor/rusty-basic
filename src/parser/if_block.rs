use super::{
    unexpected, BlockNode, ConditionalBlockNode, ExpressionNode, IfBlockNode, Parser, ParserError,
    StatementNode,
};
use crate::common::{HasLocation, Location};
use crate::lexer::{Keyword, LexemeNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_if_block(&mut self, if_pos: Location) -> Result<StatementNode, ParserError> {
        self.read_demand_whitespace("Expected whitespace after IF keyword")?;
        let if_condition = self.read_demand_expression()?;
        self.read_demand_whitespace("Expected whitespace before THEN keyword")?;
        self.read_demand_keyword(Keyword::Then)?;
        let (opt_space, next) = self.read_preserve_whitespace()?;
        match next {
            // EOL, or whitespace and then EOL
            LexemeNode::EOL(_, _) => self._consume_if_block_multi_line(if_pos, if_condition),
            _ => {
                match opt_space {
                    // whitespace and then something which wasn't EOL
                    Some(_) => self._consume_if_block_single_line(if_pos, if_condition, next),
                    // THEN was not followed by space nor EOL
                    None => unexpected("Expected space or EOL", next),
                }
            }
        }
    }

    fn _consume_if_block_single_line(
        &mut self,
        if_pos: Location,
        if_condition: ExpressionNode,
        next: LexemeNode,
    ) -> Result<StatementNode, ParserError> {
        let if_block = ConditionalBlockNode {
            condition: if_condition,
            pos: if_pos,
            statements: vec![self.demand_assignment_or_sub_call(next)?],
        };
        Ok(StatementNode::IfBlock(IfBlockNode {
            if_block: if_block,
            else_if_blocks: vec![],
            else_block: None,
        }))
    }

    fn _consume_if_block_multi_line(
        &mut self,
        if_pos: Location,
        if_condition: ExpressionNode,
    ) -> Result<StatementNode, ParserError> {
        // read if statements
        let (if_statements, mut exit_lexeme) = self._demand_block_until_else_or_else_if_or_end()?;
        let if_block = ConditionalBlockNode::new(if_pos, if_condition, if_statements);
        // parse additional elseif blocks
        let mut else_if_blocks: Vec<ConditionalBlockNode> = vec![];
        while exit_lexeme.is_keyword(Keyword::ElseIf) {
            let (else_if_condition, else_if_statements, else_if_exit_lexeme) =
                self._demand_else_if_conditional_block()?;
            else_if_blocks.push(ConditionalBlockNode::new(
                exit_lexeme.location(),
                else_if_condition,
                else_if_statements,
            ));
            exit_lexeme = else_if_exit_lexeme;
        }
        // parse else block
        let else_block: Option<BlockNode>;
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
        Ok(StatementNode::IfBlock(IfBlockNode {
            if_block: if_block,
            else_if_blocks: else_if_blocks,
            else_block: else_block,
        }))
    }

    fn _demand_else_if_conditional_block(
        &mut self,
    ) -> Result<(ExpressionNode, BlockNode, LexemeNode), ParserError> {
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
    ) -> Result<(BlockNode, LexemeNode), ParserError> {
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

    fn _demand_else_block(&mut self) -> Result<BlockNode, ParserError> {
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
    use super::*;
    use crate::parser::TopLevelTokenNode;
    use crate::{assert_sub_call, assert_top_sub_call, assert_var_expr};

    #[test]
    fn test_if() {
        let input = "IF X THEN\r\nPRINT X\r\nEND IF";
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            StatementNode::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode::new(
                    Location::new(1, 1),
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_bare_name(2, 1),
                        vec!["X".as_var_expr(2, 7)]
                    )],
                ),
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
        let program = parse(input);
        assert_eq!(2, program.len());
        match &program[0] {
            TopLevelTokenNode::Statement(StatementNode::IfBlock(i)) => {
                // if condition
                assert_var_expr!(i.if_block.condition, "X");
                // if block
                assert_eq!(1, i.if_block.statements.len());
                assert_sub_call!(i.if_block.statements[0], "PRINT", "X");
                // no else_if
                assert_eq!(0, i.else_if_blocks.len());
                // no else
                assert_eq!(i.else_block, None);
            }
            _ => panic!("unexpected"),
        };
        assert_top_sub_call!(program[1], "SYSTEM");
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
            StatementNode::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode::new(
                    Location::new(1, 1),
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_bare_name(2, 5),
                        vec!["X".as_var_expr(2, 11)]
                    )],
                ),
                else_if_blocks: vec![],
                else_block: Some(vec![StatementNode::SubCall(
                    "PRINT".as_bare_name(4, 5),
                    vec!["Y".as_var_expr(4, 11)]
                )]),
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
            StatementNode::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode::new(
                    Location::new(1, 1),
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_bare_name(2, 5),
                        vec!["X".as_var_expr(2, 11)]
                    )],
                ),
                else_if_blocks: vec![ConditionalBlockNode::new(
                    Location::new(3, 1),
                    "Y".as_var_expr(3, 8),
                    vec![StatementNode::SubCall(
                        "PRINT".as_bare_name(4, 5),
                        vec!["Y".as_var_expr(4, 11)]
                    )],
                )],
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
            StatementNode::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode::new(
                    Location::new(1, 1),
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_bare_name(2, 5),
                        vec!["X".as_var_expr(2, 11)]
                    )],
                ),
                else_if_blocks: vec![
                    ConditionalBlockNode::new(
                        Location::new(3, 1),
                        "Y".as_var_expr(3, 8),
                        vec![StatementNode::SubCall(
                            "PRINT".as_bare_name(4, 5),
                            vec!["Y".as_var_expr(4, 11)]
                        )],
                    ),
                    ConditionalBlockNode::new(
                        Location::new(5, 1),
                        "Z".as_var_expr(5, 8),
                        vec![StatementNode::SubCall(
                            "PRINT".as_bare_name(6, 5),
                            vec!["Z".as_var_expr(6, 11)]
                        )],
                    ),
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
            StatementNode::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode::new(
                    Location::new(1, 1),
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_bare_name(2, 5),
                        vec!["X".as_var_expr(2, 11)]
                    )],
                ),
                else_if_blocks: vec![ConditionalBlockNode::new(
                    Location::new(3, 1),
                    "Y".as_var_expr(3, 8),
                    vec![StatementNode::SubCall(
                        "PRINT".as_bare_name(4, 5),
                        vec!["Y".as_var_expr(4, 11)]
                    )],
                )],
                else_block: Some(vec![StatementNode::SubCall(
                    "PRINT".as_bare_name(6, 5),
                    vec!["Z".as_var_expr(6, 11)]
                )]),
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
            StatementNode::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode::new(
                    Location::new(1, 1),
                    "x".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "print".as_bare_name(2, 5),
                        vec!["x".as_var_expr(2, 11)]
                    )],
                ),
                else_if_blocks: vec![ConditionalBlockNode::new(
                    Location::new(3, 1),
                    "y".as_var_expr(3, 8),
                    vec![StatementNode::SubCall(
                        "print".as_bare_name(4, 5),
                        vec!["y".as_var_expr(4, 11)]
                    )],
                )],
                else_block: Some(vec![StatementNode::SubCall(
                    "print".as_bare_name(6, 5),
                    vec!["z".as_var_expr(6, 11)]
                )]),
            })
        );
    }
}
