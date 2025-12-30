use rusty_parser::{Print, PrintArg};

use crate::converter::common::{Context, Convertible, ConvertibleIn};
use crate::core::LintErrorPos;

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
