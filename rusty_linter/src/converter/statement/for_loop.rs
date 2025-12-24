use crate::converter::common::Context;
use crate::converter::common::Convertible;
use crate::converter::common::ConvertibleIn;
use crate::converter::common::ExprContext;
use crate::core::LintErrorPos;
use rusty_parser::ForLoop;

impl Convertible for ForLoop {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        let variable_name = self
            .variable_name
            .convert_in(ctx, ExprContext::Assignment)?;
        let lower_bound = self.lower_bound.convert_in_default(ctx)?;
        let upper_bound = self.upper_bound.convert_in_default(ctx)?;
        let step = self.step.convert_in_default(ctx)?;
        let statements = self.statements.convert(ctx)?;
        let next_counter = self.next_counter.convert_in(ctx, ExprContext::Assignment)?;
        Ok(Self {
            variable_name,
            lower_bound,
            upper_bound,
            step,
            statements,
            next_counter,
        })
    }
}
