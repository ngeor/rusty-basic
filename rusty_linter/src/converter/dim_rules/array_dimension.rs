use rusty_parser::ArrayDimension;

use crate::converter::common::{Context, Convertible, ConvertibleIn};
use crate::core::LintErrorPos;

impl Convertible for ArrayDimension {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        Ok(Self {
            lbound: self.lbound.convert_in_default(ctx)?,
            ubound: self.ubound.convert_in_default(ctx)?,
        })
    }
}
