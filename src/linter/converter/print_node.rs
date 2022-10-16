use crate::linter::converter::conversion_traits::SameTypeConverterWithImplicits;
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::parser::{PrintArg, PrintNode};

impl SameTypeConverterWithImplicits<PrintNode> for ConverterImpl {
    fn convert_same_type_with_implicits(&mut self, a: PrintNode) -> R<PrintNode> {
        let (format_string, mut implicit_vars_format_string) = self
            .context
            .on_opt_expression(a.format_string, ExprContext::Default)?;
        let (args, mut implicit_vars_args) = self.convert_same_type_with_implicits(a.args)?;

        implicit_vars_format_string.append(&mut implicit_vars_args);

        Ok((
            PrintNode {
                file_number: a.file_number,
                lpt1: a.lpt1,
                format_string,
                args,
            },
            implicit_vars_format_string,
        ))
    }
}

impl SameTypeConverterWithImplicits<PrintArg> for ConverterImpl {
    fn convert_same_type_with_implicits(&mut self, a: PrintArg) -> R<PrintArg> {
        match a {
            PrintArg::Comma => Ok((PrintArg::Comma, vec![])),
            PrintArg::Semicolon => Ok((PrintArg::Semicolon, vec![])),
            PrintArg::Expression(e) => self
                .context
                .on_expression(e, ExprContext::Default)
                .map(|(e, vars)| (PrintArg::Expression(e), vars)),
        }
    }
}
