use rusty_parser::ArrayDimension;

use crate::converter::common::{Convertible, ConvertibleIn};
use crate::core::{LintErrorPos, LinterContext};

impl Convertible for ArrayDimension {
    fn convert(self, ctx: &mut LinterContext) -> Result<Self, LintErrorPos> {
        Ok(Self {
            lbound: self.lbound.convert_in_default(ctx)?,
            ubound: self.ubound.convert_in_default(ctx)?,
        })
    }
}
