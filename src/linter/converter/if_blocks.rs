use crate::common::QErrorNode;
use crate::linter::converter::conversion_traits::SameTypeConverter;
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::{ConditionalBlockNode, IfBlockNode};

impl SameTypeConverter<ConditionalBlockNode> for ConverterImpl {
    fn convert(&mut self, a: ConditionalBlockNode) -> Result<ConditionalBlockNode, QErrorNode> {
        let condition = self
            .context
            .on_expression(a.condition, ExprContext::Default)?;
        let statements = self.convert_block_removing_constants(a.statements)?;
        Ok(ConditionalBlockNode {
            condition,
            statements,
        })
    }
}

impl SameTypeConverter<IfBlockNode> for ConverterImpl {
    fn convert(&mut self, a: IfBlockNode) -> Result<IfBlockNode, QErrorNode> {
        let if_block = self.convert(a.if_block)?;
        let else_if_blocks = self.convert(a.else_if_blocks)?;
        let else_block = self.convert(a.else_block)?;
        Ok(IfBlockNode {
            if_block,
            else_if_blocks,
            else_block,
        })
    }
}
