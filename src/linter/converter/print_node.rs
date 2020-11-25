use super::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::common::*;
use crate::linter::converter::context::ExprContext;
use crate::parser::{PrintArg, PrintNode, QualifiedNameNode};

impl<'a> ConverterWithImplicitVariables<PrintNode, PrintNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: PrintNode,
    ) -> Result<(PrintNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (format_string, mut implicit_vars_format_string) = self
            .context
            .on_opt_expression(a.format_string, ExprContext::Default)?;
        let (args, mut implicit_vars_args) = self.convert_and_collect_implicit_variables(a.args)?;

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

impl<'a> ConverterWithImplicitVariables<PrintArg, PrintArg> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: PrintArg,
    ) -> Result<(PrintArg, Vec<QualifiedNameNode>), QErrorNode> {
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
