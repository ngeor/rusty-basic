use crate::converter::common::Context;
use crate::converter::common::Convertible;
use crate::converter::common::ConvertibleIn;
use crate::core::LintErrorPos;
use rusty_parser::DoLoop;

impl Convertible for DoLoop {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        Ok(Self {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
            ..self
        })
    }
}
