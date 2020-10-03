use super::converter::{Converter, ConverterImpl};
use crate::common::*;
use crate::linter::types::Statement;
use crate::parser;

impl<'a> Converter<parser::Statement, Statement> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::Statement) -> Result<Statement, QErrorNode> {
        match a {
            parser::Statement::Comment(c) => Ok(Statement::Comment(c)),
            parser::Statement::Assignment(n, e) => self.assignment(n, e),
            parser::Statement::Const(n, e) => self.constant(n, e),
            parser::Statement::SubCall(n, args) => self.sub_call(n, args),
            parser::Statement::IfBlock(i) => Ok(Statement::IfBlock(self.convert(i)?)),
            parser::Statement::SelectCase(s) => Ok(Statement::SelectCase(self.convert(s)?)),
            parser::Statement::ForLoop(f) => Ok(Statement::ForLoop(self.convert(f)?)),
            parser::Statement::While(c) => Ok(Statement::While(self.convert(c)?)),
            parser::Statement::ErrorHandler(l) => Ok(Statement::ErrorHandler(l)),
            parser::Statement::Label(l) => Ok(Statement::Label(l)),
            parser::Statement::GoTo(l) => Ok(Statement::GoTo(l)),
            parser::Statement::Dim(dim_name_node) => {
                self.convert(dim_name_node).map(|x| Statement::Dim(x))
            }
            parser::Statement::Print(print_node) => {
                self.convert(print_node).map(|p| Statement::Print(p))
            }
        }
    }
}
