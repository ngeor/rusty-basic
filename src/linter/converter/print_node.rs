use super::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::common::*;
use crate::linter::{PrintArg, PrintNode};
use crate::parser;
use crate::parser::QualifiedNameNode;

impl<'a> ConverterWithImplicitVariables<parser::PrintNode, PrintNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: parser::PrintNode,
    ) -> Result<(PrintNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (format_string, mut implicit_vars_format_string) =
            self.convert_and_collect_implicit_variables(a.format_string)?;
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

impl<'a> ConverterWithImplicitVariables<parser::PrintArg, PrintArg> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: parser::PrintArg,
    ) -> Result<(PrintArg, Vec<QualifiedNameNode>), QErrorNode> {
        match a {
            parser::PrintArg::Comma => Ok((PrintArg::Comma, vec![])),
            parser::PrintArg::Semicolon => Ok((PrintArg::Semicolon, vec![])),
            parser::PrintArg::Expression(e) => self
                .convert_and_collect_implicit_variables(e)
                .map(|(e, vars)| (PrintArg::Expression(e), vars)),
        }
    }
}
