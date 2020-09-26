use crate::common::QErrorNode;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{CaseBlockNode, CaseExpression, SelectCaseNode};
use crate::parser;

impl<'a> Converter<parser::SelectCaseNode, SelectCaseNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::SelectCaseNode) -> Result<SelectCaseNode, QErrorNode> {
        Ok(SelectCaseNode {
            expr: self.convert(a.expr)?,
            case_blocks: self.convert(a.case_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}

impl<'a> Converter<parser::CaseBlockNode, CaseBlockNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::CaseBlockNode) -> Result<CaseBlockNode, QErrorNode> {
        Ok(CaseBlockNode {
            expr: self.convert(a.expr)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl<'a> Converter<parser::CaseExpression, CaseExpression> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::CaseExpression) -> Result<CaseExpression, QErrorNode> {
        match a {
            parser::CaseExpression::Simple(e) => self.convert(e).map(|x| CaseExpression::Simple(x)),
            parser::CaseExpression::Is(op, e) => self.convert(e).map(|x| CaseExpression::Is(op, x)),
            parser::CaseExpression::Range(from, to) => self
                .convert(from)
                .and_then(|x| self.convert(to).map(|y| CaseExpression::Range(x, y))),
        }
    }
}
