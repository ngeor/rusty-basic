use super::{
    unexpected, CaseBlockNode, CaseExpression, ExpressionNode, Operand, Parser, ParserError,
    SelectCaseNode, Statement, StatementContext, StatementNodes,
};
use crate::lexer::{Keyword, LexemeNode};
use crate::parser::buf_lexer::BufLexerUndo;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_select_case(&mut self) -> Result<Statement, ParserError> {
        // initial state: we just read the "SELECT" keyword
        self.read_demand_whitespace("Expected space after SELECT")?;
        self.read_demand_keyword(Keyword::Case)?;
        self.read_demand_whitespace("Expected space after CASE")?;
        let expr: ExpressionNode = self.read_demand_expression()?;
        let inline_comment = self.read_optional_comment()?;

        let mut case_blocks: Vec<CaseBlockNode> = vec![];
        let mut next = self.read_skipping_whitespace_and_eol()?;
        let mut has_more = true;
        let mut else_block: Option<StatementNodes> = None;
        while has_more {
            match next {
                LexemeNode::Keyword(Keyword::End, _, _) => {
                    // END SELECT
                    self.read_demand_whitespace("Expected space after END")?;
                    self.read_demand_keyword(Keyword::Select)?;
                    self.finish_line(StatementContext::Normal)?;
                    has_more = false;
                }
                LexemeNode::Keyword(Keyword::Case, _, _) => {
                    // CASE something
                    next = self.read_after_case(&mut case_blocks, &mut else_block)?;
                }
                _ => return unexpected("Expected CASE or END", next),
            }
        }

        Ok(Statement::SelectCase(SelectCaseNode {
            inline_comment,
            expr,
            case_blocks,
            else_block,
        }))
    }

    fn read_after_case(
        &mut self,
        case_blocks: &mut Vec<CaseBlockNode>,
        else_block: &mut Option<StatementNodes>,
    ) -> Result<LexemeNode, ParserError> {
        self.read_demand_whitespace("Expected space after CASE")?;
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Keyword(Keyword::Is, _, _) => {
                // CASE IS >= 5
                self.read_after_is(case_blocks)
            }
            LexemeNode::Keyword(Keyword::Else, _, _) => self.read_after_else(next, else_block),
            _ => self.read_simple(next, case_blocks),
        }
    }

    fn read_after_is(
        &mut self,
        case_blocks: &mut Vec<CaseBlockNode>,
    ) -> Result<LexemeNode, ParserError> {
        self.read_demand_whitespace("Expected space after IS")?;
        let first = self.buf_lexer.read()?;
        let op: Operand;
        match first {
            LexemeNode::Symbol('=', _) => {
                op = Operand::Equal;
            }
            LexemeNode::Symbol('<', _) => {
                op = if self.try_read_equals()? {
                    Operand::LessOrEqual
                } else {
                    Operand::Less
                }
            }
            LexemeNode::Symbol('>', _) => {
                op = if self.try_read_equals()? {
                    Operand::GreaterOrEqual
                } else {
                    Operand::Greater
                }
            }
            _ => {
                return unexpected("Expected =, < or > after IS", first);
            }
        }
        let expr = self.read_demand_expression_skipping_whitespace()?;
        self.read_case_body(CaseExpression::Is(op, expr), case_blocks)
    }

    fn try_read_equals(&mut self) -> Result<bool, ParserError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Symbol('=', _) => Ok(true),
            _ => {
                self.buf_lexer.undo(next);
                Ok(false)
            }
        }
    }

    fn read_after_else(
        &mut self,
        next: LexemeNode,
        else_block: &mut Option<StatementNodes>,
    ) -> Result<LexemeNode, ParserError> {
        // CASE ELSE
        if else_block.is_some() {
            // already set CASE ELSE...
            return unexpected("CASE ELSE already defined", next);
        }
        self.finish_line(StatementContext::Normal)?;
        let (statements, exit_lexeme) = self.parse_statements(
            |x| match x {
                LexemeNode::Keyword(Keyword::End, _, _) => true,
                _ => false,
            },
            "Unexpected EOF while looking for end of SELECT after CASE ELSE",
        )?;
        *else_block = Some(statements);
        Ok(exit_lexeme)
    }

    fn read_simple(
        &mut self,
        next: LexemeNode,
        case_blocks: &mut Vec<CaseBlockNode>,
    ) -> Result<LexemeNode, ParserError> {
        // simple: CASE 5 or range: CASE 5 TO 6
        let expr = self.demand_expression(next)?;
        let mut upper_expr: Option<ExpressionNode> = None;
        // try to read ahead a whitespace and "TO" keyword
        let next2 = self.buf_lexer.read()?;
        if next2.is_whitespace() {
            let next3 = self.buf_lexer.read()?;
            if next3.is_keyword(Keyword::To) {
                // CASE 5 TO 6
                upper_expr = Some(self.read_demand_expression_skipping_whitespace()?);
            } else {
                self.buf_lexer.undo(next3);
                self.buf_lexer.undo(next2);
            }
        } else {
            self.buf_lexer.undo(next2);
        }
        self.finish_line(StatementContext::Normal)?;
        let case_expr = match upper_expr {
            Some(u) => CaseExpression::Range(expr, u),
            None => CaseExpression::Simple(expr),
        };
        self.read_case_body(case_expr, case_blocks)
    }

    fn read_case_body(
        &mut self,
        expr: CaseExpression,
        case_blocks: &mut Vec<CaseBlockNode>,
    ) -> Result<LexemeNode, ParserError> {
        let (statements, exit_lexeme) = self.parse_statements(
            |x| match x {
                LexemeNode::Keyword(Keyword::End, _, _) => true,
                LexemeNode::Keyword(Keyword::Case, _, _) => true,
                _ => false,
            },
            "Unexpected EOF while looking for CASE or END after CASE",
        )?;
        case_blocks.push(CaseBlockNode { expr, statements });
        Ok(exit_lexeme)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::*;

    #[test]
    fn test_inline_comment() {
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
                    inline_comment: Some(" testing for x".to_string().at_rc(2, 23)),
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
}
