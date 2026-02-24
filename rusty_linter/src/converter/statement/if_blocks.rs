use rusty_parser::{ConditionalBlock, IfBlock};

use crate::converter::common::{Convertible, ConvertibleIn};
use crate::core::{LintErrorPos, LinterContext};

impl Convertible for ConditionalBlock {
    fn convert(self, ctx: &mut LinterContext) -> Result<Self, LintErrorPos> {
        Ok(Self {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
        })
    }
}

impl Convertible for IfBlock {
    fn convert(self, ctx: &mut LinterContext) -> Result<Self, LintErrorPos> {
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
