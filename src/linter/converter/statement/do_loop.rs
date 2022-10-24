use crate::common::QErrorNode;
use crate::linter::converter::context::Context;
use crate::linter::converter::traits::Convertible;
use crate::parser::DoLoopNode;

impl Convertible for DoLoopNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        Ok(Self {
            condition: self.condition.convert_in_default(ctx)?,
            statements: self.statements.convert(ctx)?,
            ..self
        })
    }
}
