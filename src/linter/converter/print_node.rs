use super::converter::{Converter, ConverterImpl};
use crate::common::*;
use crate::linter::{PrintArg, PrintNode};
use crate::parser;

impl<'a> Converter<parser::PrintNode, PrintNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::PrintNode) -> Result<PrintNode, QErrorNode> {
        Ok(PrintNode {
            file_number: a.file_number,
            lpt1: a.lpt1,
            format_string: self.convert(a.format_string)?,
            args: self.convert(a.args)?,
        })
    }
}

impl<'a> Converter<parser::PrintArg, PrintArg> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::PrintArg) -> Result<PrintArg, QErrorNode> {
        match a {
            parser::PrintArg::Comma => Ok(PrintArg::Comma),
            parser::PrintArg::Semicolon => Ok(PrintArg::Semicolon),
            parser::PrintArg::Expression(e) => self.convert(e).map(|e| PrintArg::Expression(e)),
        }
    }
}
