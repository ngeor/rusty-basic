use crate::common::*;
use crate::linter::converter::converter::{Context, Convertible};
use crate::linter::converter::expr_rules::{on_expression, on_opt_expression};
use crate::linter::converter::ExprContext;
use crate::parser::{PrintArg, PrintNode};

impl Convertible for PrintNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        Ok(Self {
            format_string: on_opt_expression(ctx, self.format_string, ExprContext::Default)?,
            args: self.args.convert(ctx)?,
            ..self
        })
    }
}

impl Convertible for PrintArg {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        match self {
            Self::Expression(e) => {
                on_expression(ctx, e, ExprContext::Default).map(Self::Expression)
            }
            _ => Ok(self),
        }
    }
}
