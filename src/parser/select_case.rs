use super::{
    unexpected, CaseBlockNode, CaseExpression, ExpressionNode, Operand, Parser, ParserError,
    SelectCaseNode, Statement, StatementNodes,
};
use crate::lexer::{Keyword, LexemeNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_select_case(&mut self) -> Result<Statement, ParserError> {
        // initial state: we just read the "SELECT" keyword
        self.read_demand_whitespace("Expected space after SELECT")?;
        self.read_demand_keyword(Keyword::Case)?;
        self.read_demand_whitespace("Expected space after CASE")?;
        let expr: ExpressionNode = self.read_demand_expression()?;
        self.read_demand_eol_skipping_whitespace()?;

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
                    self.read_demand_eol_or_eof_skipping_whitespace()?;
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
        self.read_demand_eol_skipping_whitespace()?;
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
        self.read_demand_eol_skipping_whitespace()?;
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
