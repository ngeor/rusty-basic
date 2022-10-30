use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use rusty_common::*;
use rusty_parser::{CaseBlockNode, CaseExpression, SelectCaseNode};

impl Convertible for SelectCaseNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let expr = self.expr.convert_in_default(ctx)?;
        let case_blocks = self.case_blocks.convert(ctx)?;
        let else_block = self.else_block.convert(ctx)?;
        let inline_comments = self.inline_comments;
        Ok(SelectCaseNode {
            expr,
            case_blocks,
            else_block,
            inline_comments,
        })
    }
}

impl Convertible for CaseBlockNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let expression_list = self.expression_list.convert(ctx)?;
        let statements = self.statements.convert(ctx)?;
        Ok(CaseBlockNode {
            expression_list,
            statements,
        })
    }
}

impl Convertible for CaseExpression {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        match self {
            CaseExpression::Simple(e) => e.convert_in_default(ctx).map(CaseExpression::Simple),
            CaseExpression::Is(op, e) => {
                e.convert_in_default(ctx).map(|e| CaseExpression::Is(op, e))
            }
            CaseExpression::Range(from, to) => {
                let from = from.convert_in_default(ctx)?;
                let to = to.convert_in_default(ctx)?;
                Ok(CaseExpression::Range(from, to))
            }
        }
    }
}
