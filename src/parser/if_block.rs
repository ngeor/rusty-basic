use super::{ConditionalBlockNode, ExpressionNode, IfBlockNode, Parser, StatementNode};
use crate::common::Location;
use crate::lexer::{Keyword, LexerError};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_if_block(&mut self) -> Result<Option<StatementNode>, LexerError> {
        let opt_if_pos = self.buf_lexer.try_consume_keyword(Keyword::If)?;
        if let Some(if_pos) = opt_if_pos {
            // parse main if block
            let if_block = self._demand_conditional_block(if_pos)?;

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
            Ok(Some(StatementNode::IfBlock(IfBlockNode {
                if_block: if_block,
                else_if_blocks: else_if_blocks,
                else_block: else_block,
            })))
        } else {
            Ok(None)
        }
    }

    fn _demand_conditional_block(
        &mut self,
        pos: Location,
    ) -> Result<ConditionalBlockNode, LexerError> {
        let condition = self._demand_condition()?;
        let block = self.parse_block()?;
        Ok(ConditionalBlockNode {
            condition,
            block,
            pos,
        })
    }

    fn _demand_condition(&mut self) -> Result<ExpressionNode, LexerError> {
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
    use crate::common::StripLocation;
    use crate::parser::{ConditionalBlock, Expression, IfBlock, Statement};

    #[test]
    fn test_if() {
        let input = "IF X THEN\r\nPRINT X\r\nEND IF";
        let mut parser = Parser::from(input);
        let if_block = parser
            .try_parse_if_block()
            .unwrap()
            .unwrap()
            .strip_location();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name("X"),
                    vec![sub_call("PRINT", vec![Expression::variable_name("X")],)],
                ),
                else_if_blocks: vec![],
                else_block: None,
            }),
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
        let if_block = parser
            .try_parse_if_block()
            .unwrap()
            .unwrap()
            .strip_location();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name("X"),
                    vec![sub_call("PRINT", vec![Expression::variable_name("X")],)],
                ),
                else_if_blocks: vec![],
                else_block: Some(vec![sub_call(
                    "PRINT",
                    vec![Expression::variable_name("Y")],
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
        let mut parser = Parser::from(input);
        let if_block = parser
            .try_parse_if_block()
            .unwrap()
            .unwrap()
            .strip_location();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name("X"),
                    vec![sub_call("PRINT", vec![Expression::variable_name("X")],)],
                ),
                else_if_blocks: vec![ConditionalBlock::new(
                    Expression::variable_name("Y"),
                    vec![sub_call("PRINT", vec![Expression::variable_name("Y")],)],
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
        let mut parser = Parser::from(input);
        let if_block = parser
            .try_parse_if_block()
            .unwrap()
            .unwrap()
            .strip_location();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name("X"),
                    vec![sub_call("PRINT", vec![Expression::variable_name("X")],)],
                ),
                else_if_blocks: vec![
                    ConditionalBlock::new(
                        Expression::variable_name("Y"),
                        vec![sub_call("PRINT", vec![Expression::variable_name("Y")],)],
                    ),
                    ConditionalBlock::new(
                        Expression::variable_name("Z"),
                        vec![sub_call("PRINT", vec![Expression::variable_name("Z")],)],
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
        let mut parser = Parser::from(input);
        let if_block = parser
            .try_parse_if_block()
            .unwrap()
            .unwrap()
            .strip_location();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name("X"),
                    vec![sub_call("PRINT", vec![Expression::variable_name("X")],)],
                ),
                else_if_blocks: vec![ConditionalBlock::new(
                    Expression::variable_name("Y"),
                    vec![sub_call("PRINT", vec![Expression::variable_name("Y")],)],
                )],
                else_block: Some(vec![sub_call(
                    "PRINT",
                    vec![Expression::variable_name("Z")],
                )]),
            }),
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
        let mut parser = Parser::from(input);
        let if_block = parser
            .try_parse_if_block()
            .unwrap()
            .unwrap()
            .strip_location();
        assert_eq!(
            if_block,
            Statement::IfBlock(IfBlock {
                if_block: ConditionalBlock::new(
                    Expression::variable_name("x"),
                    vec![sub_call("print", vec![Expression::variable_name("x")],)],
                ),
                else_if_blocks: vec![ConditionalBlock::new(
                    Expression::variable_name("y"),
                    vec![sub_call("print", vec![Expression::variable_name("y")],)],
                )],
                else_block: Some(vec![sub_call(
                    "print",
                    vec![Expression::variable_name("z")],
                )]),
            }),
        );
    }
}
