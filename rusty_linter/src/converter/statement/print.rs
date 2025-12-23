use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use crate::error::LintErrorPos;
use rusty_parser::specific::{Print, PrintArg};

impl Convertible for Print {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        Ok(Self {
            format_string: self.format_string.convert_in_default(ctx)?,
            args: self.args.convert(ctx)?,
            ..self
        })
    }
}

impl Convertible for PrintArg {
    fn convert(self, ctx: &mut Context) -> Result<Self, LintErrorPos> {
        match self {
            Self::Expression(e) => e.convert_in_default(ctx).map(Self::Expression),
            _ => Ok(self),
        }
    }
}
