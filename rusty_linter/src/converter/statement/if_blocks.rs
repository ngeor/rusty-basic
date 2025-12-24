use crate::converter::common::Context;
use crate::converter::common::Convertible;
use crate::core::LintErrorPos;
use rusty_parser::specific::{ConditionalBlock, IfBlock};

impl Convertible for ConditionalBlock {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        Ok(Self {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
        })
    }
}

impl Convertible for IfBlock {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        let if_block = self.if_block.convert(ctx)?;
        let else_if_blocks = self.else_if_blocks.convert(ctx)?;
        let else_block = self.else_block.convert(ctx)?;
        Ok(Self {
            if_block,
            else_if_blocks,
            else_block,
        })
    }
}
