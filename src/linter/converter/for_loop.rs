use crate::common::*;
use crate::linter::converter::converter::Convertible;
use crate::linter::converter::expr_rules::{on_expression, on_opt_expression};
use crate::linter::converter::{Context, ExprContext};
use crate::parser::ForLoopNode;

impl Convertible for ForLoopNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let variable_name = on_expression(ctx, self.variable_name, ExprContext::Assignment)?;
        let lower_bound = on_expression(ctx, self.lower_bound, ExprContext::Default)?;
        let upper_bound = on_expression(ctx, self.upper_bound, ExprContext::Default)?;
        let step = on_opt_expression(ctx, self.step, ExprContext::Default)?;
        let statements = self.statements.convert(ctx)?;
        let next_counter = on_opt_expression(ctx, self.next_counter, ExprContext::Assignment)?;
        Ok(ForLoopNode {
            variable_name,
            lower_bound,
            upper_bound,
            step,
            statements,
            next_counter,
        })
    }
}
