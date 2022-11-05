use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use rusty_common::QErrorPos;
use rusty_parser::DoLoop;

impl Convertible for DoLoop {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorPos> {
        Ok(Self {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
            ..self
        })
    }
}
