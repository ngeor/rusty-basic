use crate::common::QErrorNode;
use crate::linter::converter::converter::Convertible;
use crate::linter::converter::expr_rules::on_expression;
use crate::linter::converter::{Context, ExprContext};
use crate::parser::DoLoopNode;

impl Convertible for DoLoopNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        Ok(Self {
            condition: on_expression(ctx, self.condition, ExprContext::Default)?,
            statements: self.statements.convert(ctx)?,
            ..self
        })
    }
}
