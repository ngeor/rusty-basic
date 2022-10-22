use crate::common::*;
use crate::linter::converter::converter::Convertible;
use crate::linter::converter::expr_rules::on_expression;
use crate::linter::converter::{Context, ExprContext};
use crate::parser::{CaseBlockNode, CaseExpression, SelectCaseNode};

impl Convertible for SelectCaseNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let expr = on_expression(ctx, self.expr, ExprContext::Default)?;
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
            CaseExpression::Simple(e) => {
                on_expression(ctx, e, ExprContext::Default).map(CaseExpression::Simple)
            }
            CaseExpression::Is(op, e) => {
                on_expression(ctx, e, ExprContext::Default).map(|e| CaseExpression::Is(op, e))
            }
            CaseExpression::Range(from, to) => {
                let from = on_expression(ctx, from, ExprContext::Default)?;
                let to = on_expression(ctx, to, ExprContext::Default)?;
                Ok(CaseExpression::Range(from, to))
            }
        }
    }
}
