use crate::common::*;
use crate::linter::converter::converter::Context;
use crate::linter::converter::traits::Convertible;
use crate::parser::{ConditionalBlockNode, IfBlockNode};

impl Convertible for ConditionalBlockNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        Ok(ConditionalBlockNode {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
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
