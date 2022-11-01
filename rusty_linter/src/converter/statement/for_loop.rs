use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use crate::converter::types::ExprContext;
use rusty_common::*;
use rusty_parser::ForLoopNode;

impl Convertible for ForLoopNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        let variable_name = self
            .variable_name
            .convert_in(ctx, ExprContext::Assignment)?;
        let lower_bound = self.lower_bound.convert_in_default(ctx)?;
        let upper_bound = self.upper_bound.convert_in_default(ctx)?;
        let step = self.step.convert_in_default(ctx)?;
        let statements = self.statements.convert(ctx)?;
        let next_counter = self.next_counter.convert_in(ctx, ExprContext::Assignment)?;
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