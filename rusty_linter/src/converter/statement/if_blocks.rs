use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use rusty_common::*;
use rusty_parser::{ConditionalBlock, IfBlock};

impl Convertible for ConditionalBlock {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorPos> {
        Ok(ConditionalBlock {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
        })
    }
}

impl Convertible for IfBlock {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorPos> {
        let if_block = self.if_block.convert(ctx)?;
        let else_if_blocks = self.else_if_blocks.convert(ctx)?;
        let else_block = self.else_block.convert(ctx)?;
        Ok(IfBlock {
            if_block,
            else_if_blocks,
            else_block,
        })
    }
}
