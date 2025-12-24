use crate::converter::common::Context;
use crate::converter::common::Convertible;
use crate::converter::common::ConvertibleIn;
use crate::core::LintErrorPos;
use rusty_parser::ArrayDimension;

impl Convertible for ArrayDimension {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        Ok(Self {
            lbound: self.lbound.convert_in_default(ctx)?,
            ubound: self.ubound.convert_in_default(ctx)?,
        })
    }
}
