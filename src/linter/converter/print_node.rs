use crate::common::QErrorNode;
use crate::linter::converter::conversion_traits::SameTypeConverter;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::{PrintArg, PrintNode};

impl SameTypeConverter<PrintNode> for ConverterImpl {
    fn convert(&mut self, a: PrintNode) -> Result<PrintNode, QErrorNode> {
        let format_string = self
            .context
            .on_opt_expression(a.format_string, ExprContext::Default)?;
        let args = self.convert(a.args)?;

        Ok(PrintNode {
            file_number: a.file_number,
            lpt1: a.lpt1,
            format_string,
            args,
        })
    }
}

impl SameTypeConverter<PrintArg> for ConverterImpl {
    fn convert(&mut self, a: PrintArg) -> Result<PrintArg, QErrorNode> {
        match a {
            PrintArg::Expression(e) => self
                .context
                .on_expression(e, ExprContext::Default)
                .map(|e| PrintArg::Expression(e)),
            _ => Ok(a),
        }
    }
}
