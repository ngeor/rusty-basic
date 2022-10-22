use crate::common::QErrorNode;
use crate::linter::converter::converter::Context;
use crate::linter::converter::traits::Convertible;
use crate::parser::ArrayDimension;

impl Convertible for ArrayDimension {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        Ok(Self {
            lbound: self.lbound.convert_in_default(ctx)?,
            ubound: self.ubound.convert_in_default(ctx)?,
        })
    }
}
