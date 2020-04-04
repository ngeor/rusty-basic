use super::*;
use std::io::BufRead;

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalBlock {
    pub condition: Expression,
    pub block: Block,
}

impl ConditionalBlock {
    pub fn new(condition: Expression, block: Block) -> ConditionalBlock {
        ConditionalBlock { condition, block }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBlock {
    pub if_block: ConditionalBlock,
    pub else_if_blocks: Vec<ConditionalBlock>,
    pub else_block: Option<Block>,
}

impl IfBlock {
    #[cfg(test)]
    pub fn new_if_else(condition: Expression, if_block: Block, else_block: Block) -> IfBlock {
        IfBlock {
            if_block: ConditionalBlock::new(condition, if_block),
            else_if_blocks: vec![],
            else_block: Some(else_block),
        }
    }
}

impl<T: BufRead> Parser<T> {
    pub fn try_parse_if_block(&mut self) -> Result<Option<Statement>> {
        if self.buf_lexer.try_consume_word("IF")? {
            // parse main if block
            let if_block = self._demand_conditional_block()?;
            // parse additional elseif blocks
            let mut else_if_blocks: Vec<ConditionalBlock> = vec![];
            while self.buf_lexer.try_consume_word("ELSEIF")? {
                else_if_blocks.push(self._demand_conditional_block()?);
            }
            // parse else block
            let else_block = if self.buf_lexer.try_consume_word("ELSE")? {
                self.buf_lexer.demand_eol()?;
                Some(self.parse_block()?)
            } else {
                None
            };
            // parse end if
            self.buf_lexer.demand_specific_word("END")?;
            self.buf_lexer.demand_whitespace()?;
            self.buf_lexer.demand_specific_word("IF")?;
            self.buf_lexer.demand_eol_or_eof()?;
            Ok(Some(Statement::IfBlock(IfBlock {
                if_block: if_block,
                else_if_blocks: else_if_blocks,
                else_block: else_block,
            })))
        } else {
            Ok(None)
        }
    }

    fn _demand_conditional_block(&mut self) -> Result<ConditionalBlock> {
        let condition = self._demand_condition()?;
        let block = self.parse_block()?;
        Ok(ConditionalBlock::new(condition, block))
    }

    fn _demand_condition(&mut self) -> Result<Expression> {
        self.buf_lexer.demand_whitespace()?;
        let condition = self.demand_expression()?;
        self.buf_lexer.demand_whitespace()?;
        self.buf_lexer.demand_specific_word("THEN")?;
        self.buf_lexer.demand_eol()?;
        Ok(condition)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;

    #[test]
    fn test_if() {
        let input = "IF X THEN\r\nPRINT X\r\nEND IF";
        let mut parser = Parser::from(input);
        let if_block = parser.try_parse_if_block().unwrap().unwrap();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name_unqualified("X"),
                    vec![sub_call(
                        "PRINT",
                        vec![Expression::variable_name_unqualified("X")]
                    )]
                ),
                else_if_blocks: vec![],
                else_block: None
            })
        );
    }

    #[test]
    fn test_if_else() {
        let input = r#"IF X THEN
    PRINT X
ELSE
    PRINT Y
END IF"#;
        let mut parser = Parser::from(input);
        let if_block = parser.try_parse_if_block().unwrap().unwrap();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name_unqualified("X"),
                    vec![sub_call(
                        "PRINT",
                        vec![Expression::variable_name_unqualified("X")]
                    )]
                ),
                else_if_blocks: vec![],
                else_block: Some(vec![sub_call(
                    "PRINT",
                    vec![Expression::variable_name_unqualified("Y")]
                )])
            })
        );
    }

    #[test]
    fn test_if_else_if() {
        let input = r#"IF X THEN
    PRINT X
ELSEIF Y THEN
    PRINT Y
END IF"#;
        let mut parser = Parser::from(input);
        let if_block = parser.try_parse_if_block().unwrap().unwrap();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name_unqualified("X"),
                    vec![sub_call(
                        "PRINT",
                        vec![Expression::variable_name_unqualified("X")]
                    )]
                ),
                else_if_blocks: vec![ConditionalBlock::new(
                    Expression::variable_name_unqualified("Y"),
                    vec![sub_call(
                        "PRINT",
                        vec![Expression::variable_name_unqualified("Y")]
                    )]
                )],
                else_block: None
            })
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
        let mut parser = Parser::from(input);
        let if_block = parser.try_parse_if_block().unwrap().unwrap();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name_unqualified("X"),
                    vec![sub_call(
                        "PRINT",
                        vec![Expression::variable_name_unqualified("X")]
                    )]
                ),
                else_if_blocks: vec![
                    ConditionalBlock::new(
                        Expression::variable_name_unqualified("Y"),
                        vec![sub_call(
                            "PRINT",
                            vec![Expression::variable_name_unqualified("Y")]
                        )]
                    ),
                    ConditionalBlock::new(
                        Expression::variable_name_unqualified("Z"),
                        vec![sub_call(
                            "PRINT",
                            vec![Expression::variable_name_unqualified("Z")]
                        )]
                    )
                ],
                else_block: None
            })
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
        let mut parser = Parser::from(input);
        let if_block = parser.try_parse_if_block().unwrap().unwrap();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name_unqualified("X"),
                    vec![sub_call(
                        "PRINT",
                        vec![Expression::variable_name_unqualified("X")]
                    )]
                ),
                else_if_blocks: vec![ConditionalBlock::new(
                    Expression::variable_name_unqualified("Y"),
                    vec![sub_call(
                        "PRINT",
                        vec![Expression::variable_name_unqualified("Y")]
                    )]
                )],
                else_block: Some(vec![sub_call(
                    "PRINT",
                    vec![Expression::variable_name_unqualified("Z")]
                )])
            })
        );
    }
}
