use crate::linter::converter::conversion_traits::SameTypeConverterWithImplicits;
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::parser::{CaseBlockNode, CaseExpression, SelectCaseNode};

impl SameTypeConverterWithImplicits<SelectCaseNode> for ConverterImpl {
    fn convert_same_type_with_implicits(&mut self, a: SelectCaseNode) -> R<SelectCaseNode> {
        let (expr, mut implicit_vars_expr) =
            self.context.on_expression(a.expr, ExprContext::Default)?;
        let (case_blocks, mut implicit_vars_case_blocks) =
            self.convert_same_type_with_implicits(a.case_blocks)?;
        implicit_vars_expr.append(&mut implicit_vars_case_blocks);
        let (else_block, mut implicits_else) =
            self.convert_same_type_with_implicits(a.else_block)?;
        implicit_vars_expr.append(&mut implicits_else);
        Ok((
            SelectCaseNode {
                expr,
                case_blocks,
                else_block,
                inline_comments: a.inline_comments,
            },
            implicit_vars_expr,
        ))
    }
}

impl SameTypeConverterWithImplicits<CaseBlockNode> for ConverterImpl {
    fn convert_same_type_with_implicits(&mut self, a: CaseBlockNode) -> R<CaseBlockNode> {
        let (expression_list, mut implicit_vars_expr) =
            self.convert_same_type_with_implicits(a.expression_list)?;
        let (statements, mut implicits_block) =
            self.convert_block_keeping_implicits(a.statements)?;
        implicit_vars_expr.append(&mut implicits_block);
        Ok((
            CaseBlockNode {
                expression_list,
                statements,
            },
            implicit_vars_expr,
        ))
    }
}

impl SameTypeConverterWithImplicits<CaseExpression> for ConverterImpl {
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
