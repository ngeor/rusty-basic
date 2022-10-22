use crate::common::*;
use crate::linter::converter::converter::Convertible;
use crate::linter::converter::expr_rules::on_expression;
use crate::linter::converter::{Context, ExprContext};
use crate::parser::{ConditionalBlockNode, IfBlockNode};

impl Convertible for ConditionalBlockNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let condition = on_expression(ctx, self.condition, ExprContext::Default)?;
        let statements = self.statements.convert(ctx)?;
        Ok(ConditionalBlockNode {
            condition,
            statements,
        })
    }
}

impl Convertible for IfBlockNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let if_block = self.if_block.convert(ctx)?;
        let else_if_blocks = self.else_if_blocks.convert(ctx)?;
        let else_block = self.else_block.convert(ctx)?;
        Ok(IfBlockNode {
            if_block,
            else_if_blocks,
            else_block,
        })
    }
}
