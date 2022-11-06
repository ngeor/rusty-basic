use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use crate::error::LintErrorPos;
use rusty_parser::{CaseBlock, CaseExpression, SelectCase};

impl Convertible for SelectCase {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        let expr = self.expr.convert_in_default(ctx)?;
        let case_blocks = self.case_blocks.convert(ctx)?;
        let else_block = self.else_block.convert(ctx)?;
        let inline_comments = self.inline_comments;
        Ok(SelectCase {
            expr,
            case_blocks,
            else_block,
            inline_comments,
        })
    }
}

impl Convertible for CaseBlock {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        let expression_list = self.expression_list.convert(ctx)?;
        let statements = self.statements.convert(ctx)?;
        Ok(CaseBlock {
            expression_list,
            statements,
        })
    }
}

impl Convertible for CaseExpression {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
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
