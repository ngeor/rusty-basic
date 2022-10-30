use crate::linter::converter::context::Context;
use crate::linter::converter::traits::Convertible;
use rusty_common::*;
use rusty_parser::{PrintArg, PrintNode};

impl Convertible for PrintNode {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        Ok(Self {
            format_string: self.format_string.convert_in_default(ctx)?,
            args: self.args.convert(ctx)?,
            ..self
        })
    }
}

impl Convertible for PrintArg {
    fn convert(self, ctx: &mut Context) -> Result<Self, QErrorNode> {
        match self {
            Self::Expression(e) => e.convert_in_default(ctx).map(Self::Expression),
            _ => Ok(self),
        }
    }
}
