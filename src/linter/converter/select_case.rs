use crate::common::QErrorNode;
use crate::linter::converter::converter::{
    Converter, ConverterImpl, ConverterWithImplicitVariables,
};
use crate::linter::{CaseBlockNode, CaseExpression, SelectCaseNode};
use crate::parser;
use crate::parser::QualifiedNameNode;

impl<'a> ConverterWithImplicitVariables<parser::SelectCaseNode, SelectCaseNode>
    for ConverterImpl<'a>
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: parser::SelectCaseNode,
    ) -> Result<(SelectCaseNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (expr, mut implicit_vars_expr) = self.convert_and_collect_implicit_variables(a.expr)?;
        let (case_blocks, mut implicit_vars_case_blocks) =
            self.convert_and_collect_implicit_variables(a.case_blocks)?;
        implicit_vars_expr.append(&mut implicit_vars_case_blocks);
        Ok((
            SelectCaseNode {
                expr,
                case_blocks,
                else_block: self.convert(a.else_block)?,
                inline_comments: a.inline_comments,
            },
            implicit_vars_expr,
        ))
    }
}

impl<'a> ConverterWithImplicitVariables<parser::CaseBlockNode, CaseBlockNode>
    for ConverterImpl<'a>
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: parser::CaseBlockNode,
    ) -> Result<(CaseBlockNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (expr, implicit_vars_expr) = self.convert_and_collect_implicit_variables(a.expr)?;
        Ok((
            CaseBlockNode {
                expr,
                statements: self.convert(a.statements)?,
            },
            implicit_vars_expr,
        ))
    }
}

impl<'a> ConverterWithImplicitVariables<parser::CaseExpression, CaseExpression>
    for ConverterImpl<'a>
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: parser::CaseExpression,
    ) -> Result<(CaseExpression, Vec<QualifiedNameNode>), QErrorNode> {
        match a {
            parser::CaseExpression::Simple(e) => self
                .convert_and_collect_implicit_variables(e)
                .map(|(expr, implicit_vars)| (CaseExpression::Simple(expr), implicit_vars)),
            parser::CaseExpression::Is(op, e) => self
                .convert_and_collect_implicit_variables(e)
                .map(|(expr, implicit_vars)| (CaseExpression::Is(op, expr), implicit_vars)),
            parser::CaseExpression::Range(from, to) => {
                let (from, mut implicit_vars_from) =
                    self.convert_and_collect_implicit_variables(from)?;
                let (to, mut implicit_vars_to) = self.convert_and_collect_implicit_variables(to)?;
                implicit_vars_from.append(&mut implicit_vars_to);
                Ok((CaseExpression::Range(from, to), implicit_vars_from))
            }
        }
    }
}
