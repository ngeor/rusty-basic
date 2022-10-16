use crate::common::QErrorNode;
use crate::linter::converter::conversion_traits::SameTypeConverter;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::{CaseBlockNode, CaseExpression, SelectCaseNode};

impl SameTypeConverter<SelectCaseNode> for ConverterImpl {
    fn convert(&mut self, a: SelectCaseNode) -> Result<SelectCaseNode, QErrorNode> {
        let expr = self.context.on_expression(a.expr, ExprContext::Default)?;
        let case_blocks = self.convert(a.case_blocks)?;
        let else_block = self.convert(a.else_block)?;
        Ok(SelectCaseNode {
            expr,
            case_blocks,
            else_block,
            inline_comments: a.inline_comments,
        })
    }
}

impl SameTypeConverter<CaseBlockNode> for ConverterImpl {
    fn convert(&mut self, a: CaseBlockNode) -> Result<CaseBlockNode, QErrorNode> {
        let expression_list = self.convert(a.expression_list)?;
        let statements = self.convert_block_removing_constants(a.statements)?;
        Ok(CaseBlockNode {
            expression_list,
            statements,
        })
    }
}

impl SameTypeConverter<CaseExpression> for ConverterImpl {
    fn convert(&mut self, a: CaseExpression) -> Result<CaseExpression, QErrorNode> {
        match a {
            CaseExpression::Simple(e) => self
                .context
                .on_expression(e, ExprContext::Default)
                .map(CaseExpression::Simple),
            CaseExpression::Is(op, e) => self
                .context
                .on_expression(e, ExprContext::Default)
                .map(|expr| CaseExpression::Is(op, expr)),
            CaseExpression::Range(from, to) => {
                let from = self.context.on_expression(from, ExprContext::Default)?;
                let to = self.context.on_expression(to, ExprContext::Default)?;
                Ok(CaseExpression::Range(from, to))
            }
        }
    }
}
