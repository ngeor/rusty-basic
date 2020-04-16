
use super::{
    ConditionalBlockNode, ExpressionNode, IfBlockNode, Parser, ParserError, StatementNode,
};
use crate::common::{Location, ResultOptionHelper};
use crate::lexer::Keyword;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_if_block(&mut self) -> Result<Option<StatementNode>, ParserError> {
        self.buf_lexer
            .try_consume_keyword(Keyword::If)
            .opt_map(|x| self._consume_if_block(x))
    }

    fn _consume_if_block(&mut self, if_pos: Location) -> Result<StatementNode, ParserError> {
        // parse main if block
        self.buf_lexer.demand_whitespace()?;
        let if_condition = self.demand_expression()?;
        self.buf_lexer.demand_whitespace()?;
        self.buf_lexer.demand_keyword(Keyword::Then)?;
        let found_whitespace_after_then: bool = self.buf_lexer.skip_whitespace()?;
        let found_eol: bool = self.buf_lexer.try_consume_eol()?;

        if found_eol {
            self._consume_if_block_multi_line(if_pos, if_condition)
        } else {
            if found_whitespace_after_then {
                self._consume_if_block_single_line(if_pos, if_condition)
            } else {
                Err(ParserError::Unexpected(
                    "Expected statement after THEN".to_string(),
                    self.buf_lexer.read()?,
                ))
            }
        }
    }

    fn _consume_if_block_single_line(
        &mut self,
        if_pos: Location,
        if_condition: ExpressionNode,
    ) -> Result<StatementNode, ParserError> {
        let if_block = ConditionalBlockNode {
            condition: if_condition,
            pos: if_pos,
            block: vec![self.demand_single_line_statement()?],
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
        let if_block = ConditionalBlockNode::new(if_condition, self.parse_block()?, if_pos);

        // parse additional elseif blocks
        let mut else_if_blocks: Vec<ConditionalBlockNode> = vec![];
        loop {
            let opt_else_if_pos = self.buf_lexer.try_consume_keyword(Keyword::ElseIf)?;
            if let Some(else_if_pos) = opt_else_if_pos {
                else_if_blocks.push(self._demand_conditional_block(else_if_pos)?);
            } else {
                break;
            }
        }

        // parse else block
        let else_block = if self.buf_lexer.try_consume_keyword(Keyword::Else)?.is_some() {
            self.buf_lexer.demand_eol()?;
            Some(self.parse_block()?)
        } else {
            None
        };
        // parse end if
        self.buf_lexer.demand_keyword(Keyword::End)?;
        self.buf_lexer.demand_whitespace()?;
        self.buf_lexer.demand_keyword(Keyword::If)?;
        self.buf_lexer.demand_eol_or_eof()?;
        Ok(StatementNode::IfBlock(IfBlockNode {
            if_block: if_block,
            else_if_blocks: else_if_blocks,
            else_block: else_block,
        }))
    }

    fn _demand_conditional_block(
        &mut self,
        pos: Location,
    ) -> Result<ConditionalBlockNode, ParserError> {
        let condition = self._demand_condition()?;
        let block = self.parse_block()?;
        Ok(ConditionalBlockNode {
            condition,
            block,
            pos,
        })
    }

    fn _demand_condition(&mut self) -> Result<ExpressionNode, ParserError> {
        self.buf_lexer.demand_whitespace()?;
        let condition = self.demand_expression()?;
        self.buf_lexer.demand_whitespace()?;
        self.buf_lexer.demand_keyword(Keyword::Then)?;
        self.buf_lexer.demand_eol()?;
        Ok(condition)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::{assert_var_expr, assert_top_sub_call, assert_sub_call};
    use crate::parser::{TopLevelTokenNode};

    #[test]
    fn test_if() {
        let input = "IF X THEN\r\nPRINT X\r\nEND IF";
        let if_block = parse(input).demand_single_statement();
        assert_eq!(
            if_block,
            StatementNode::IfBlock(IfBlockNode {
                if_block: ConditionalBlockNode::new(
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_name(2, 1),
                        vec!["X".as_var_expr(2, 7)]
                    )],
                    Location::new(1, 1)
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
                assert_eq!(1, i.if_block.block.len());
                assert_sub_call!(i.if_block.block[0], "PRINT", "X");
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
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_name(2, 5),
                        vec!["X".as_var_expr(2, 11)]
                    )],
                    Location::new(1, 1)
                ),
                else_if_blocks: vec![],
                else_block: Some(vec![StatementNode::SubCall(
                    "PRINT".as_name(4, 5),
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
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_name(2, 5),
                        vec!["X".as_var_expr(2, 11)]
                    )],
                    Location::new(1, 1)
                ),
                else_if_blocks: vec![ConditionalBlockNode::new(
                    "Y".as_var_expr(3, 8),
                    vec![StatementNode::SubCall(
                        "PRINT".as_name(4, 5),
                        vec!["Y".as_var_expr(4, 11)]
                    )],
                    Location::new(3, 1)
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
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_name(2, 5),
                        vec!["X".as_var_expr(2, 11)]
                    )],
                    Location::new(1, 1)
                ),
                else_if_blocks: vec![
                    ConditionalBlockNode::new(
                        "Y".as_var_expr(3, 8),
                        vec![StatementNode::SubCall(
                            "PRINT".as_name(4, 5),
                            vec!["Y".as_var_expr(4, 11)]
                        )],
                        Location::new(3, 1)
                    ),
                    ConditionalBlockNode::new(
                        "Z".as_var_expr(5, 8),
                        vec![StatementNode::SubCall(
                            "PRINT".as_name(6, 5),
                            vec!["Z".as_var_expr(6, 11)]
                        )],
                        Location::new(5, 1)
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
                    "X".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "PRINT".as_name(2, 5),
                        vec!["X".as_var_expr(2, 11)]
                    )],
                    Location::new(1, 1)
                ),
                else_if_blocks: vec![ConditionalBlockNode::new(
                    "Y".as_var_expr(3, 8),
                    vec![StatementNode::SubCall(
                        "PRINT".as_name(4, 5),
                        vec!["Y".as_var_expr(4, 11)]
                    )],
                    Location::new(3, 1)
                )],
                else_block: Some(vec![StatementNode::SubCall(
                    "PRINT".as_name(6, 5),
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
                    "x".as_var_expr(1, 4),
                    vec![StatementNode::SubCall(
                        "print".as_name(2, 5),
                        vec!["x".as_var_expr(2, 11)]
                    )],
                    Location::new(1, 1)
                ),
                else_if_blocks: vec![ConditionalBlockNode::new(
                    "y".as_var_expr(3, 8),
                    vec![StatementNode::SubCall(
                        "print".as_name(4, 5),
                        vec!["y".as_var_expr(4, 11)]
                    )],
                    Location::new(3, 1)
                )],
                else_block: Some(vec![StatementNode::SubCall(
                    "print".as_name(6, 5),
                    vec!["z".as_var_expr(6, 11)]
                )]),
            })
        );
    }
}
