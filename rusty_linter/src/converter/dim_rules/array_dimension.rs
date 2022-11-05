use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use rusty_common::QErrorPos;
use rusty_parser::ArrayDimension;

impl Convertible for ArrayDimension {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorPos> {
        Ok(Self {
            lbound: self.lbound.convert_in_default(ctx)?,
            ubound: self.ubound.convert_in_default(ctx)?,
        })
    }
}
