use rusty_parser::DoLoop;

use crate::converter::common::{Convertible, ConvertibleIn};
use crate::core::{LintErrorPos, LinterContext};

impl Convertible for DoLoop {
    fn convert(self, ctx: &mut LinterContext) -> Result<Self, LintErrorPos> {
        Ok(Self {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
            ..self
        })
    }
}
