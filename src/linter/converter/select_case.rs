use crate::linter::converter::conversion_traits::{
    SameTypeConverter, SameTypeConverterWithImplicits,
};
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::parser::{CaseBlockNode, CaseExpression, SelectCaseNode};

impl<'a> SameTypeConverterWithImplicits<SelectCaseNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(&mut self, a: SelectCaseNode) -> R<SelectCaseNode> {
        let (expr, mut implicit_vars_expr) =
            self.context.on_expression(a.expr, ExprContext::Default)?;
        let (case_blocks, mut implicit_vars_case_blocks) =
            self.convert_same_type_with_implicits(a.case_blocks)?;
        implicit_vars_expr.append(&mut implicit_vars_case_blocks);
        Ok((
            SelectCaseNode {
                expr,
                case_blocks,
                else_block: self.convert_same_type(a.else_block)?,
                inline_comments: a.inline_comments,
            },
            implicit_vars_expr,
        ))
    }
}

impl<'a> SameTypeConverterWithImplicits<CaseBlockNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(&mut self, a: CaseBlockNode) -> R<CaseBlockNode> {
        let (expression_list, implicit_vars_expr) =
            self.convert_same_type_with_implicits(a.expression_list)?;
        Ok((
            CaseBlockNode {
                expression_list,
                statements: self.convert_same_type(a.statements)?,
            },
            implicit_vars_expr,
        ))
    }
}

impl<'a> SameTypeConverterWithImplicits<CaseExpression> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(&mut self, a: CaseExpression) -> R<CaseExpression> {
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
