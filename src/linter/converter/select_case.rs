use crate::linter::converter::context::ExprContext;
use crate::linter::converter::{Converter, ConverterImpl, ConverterWithImplicitVariables, R};
use crate::parser::{CaseBlockNode, CaseExpression, SelectCaseNode};

impl<'a> ConverterWithImplicitVariables<SelectCaseNode, SelectCaseNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(&mut self, a: SelectCaseNode) -> R<SelectCaseNode> {
        let (expr, mut implicit_vars_expr) =
            self.context.on_expression(a.expr, ExprContext::Default)?;
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

impl<'a> ConverterWithImplicitVariables<CaseBlockNode, CaseBlockNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(&mut self, a: CaseBlockNode) -> R<CaseBlockNode> {
        let (expression_list, implicit_vars_expr) =
            self.convert_and_collect_implicit_variables(a.expression_list)?;
        Ok((
            CaseBlockNode {
                expression_list,
                statements: self.convert(a.statements)?,
            },
            implicit_vars_expr,
        ))
    }
}

impl<'a> ConverterWithImplicitVariables<CaseExpression, CaseExpression> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(&mut self, a: CaseExpression) -> R<CaseExpression> {
        match a {
            CaseExpression::Simple(e) => self
                .context
                .on_expression(e, ExprContext::Default)
                .map(|(expr, implicit_vars)| (CaseExpression::Simple(expr), implicit_vars)),
            CaseExpression::Is(op, e) => self
                .context
                .on_expression(e, ExprContext::Default)
                .map(|(expr, implicit_vars)| (CaseExpression::Is(op, expr), implicit_vars)),
            CaseExpression::Range(from, to) => {
                let (from, mut implicit_vars_from) =
                    self.context.on_expression(from, ExprContext::Default)?;
                let (to, mut implicit_vars_to) =
                    self.context.on_expression(to, ExprContext::Default)?;
                implicit_vars_from.append(&mut implicit_vars_to);
                Ok((CaseExpression::Range(from, to), implicit_vars_from))
            }
        }
    }
}
