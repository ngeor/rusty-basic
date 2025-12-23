use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use crate::error::LintErrorPos;
use rusty_parser::specific::DoLoop;

impl Convertible for DoLoop {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        Ok(Self {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
            ..self
        })
    }
}
