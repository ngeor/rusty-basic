use crate::linter::converter::conversion_traits::{
    SameTypeConverter, SameTypeConverterWithImplicits,
};
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::parser::{ConditionalBlockNode, IfBlockNode};

impl<'a> SameTypeConverterWithImplicits<ConditionalBlockNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(
        &mut self,
        a: ConditionalBlockNode,
    ) -> R<ConditionalBlockNode> {
        let (condition, implicit_vars) = self
            .context
            .on_expression(a.condition, ExprContext::Default)?;
        Ok((
            ConditionalBlockNode {
                condition,
                statements: self.convert_same_type(a.statements)?,
            },
            implicit_vars,
        ))
    }
}

impl<'a> SameTypeConverterWithImplicits<IfBlockNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(&mut self, a: IfBlockNode) -> R<IfBlockNode> {
        let (if_block, mut implicit_vars_if_block) =
            self.convert_same_type_with_implicits(a.if_block)?;
        let (else_if_blocks, mut implicit_vars_else_if_blocks) =
            self.convert_same_type_with_implicits(a.else_if_blocks)?;

        implicit_vars_if_block.append(&mut implicit_vars_else_if_blocks);

        Ok((
            IfBlockNode {
                if_block,
                else_if_blocks,
                else_block: self.convert_same_type(a.else_block)?,
            },
            implicit_vars_if_block,
        ))
    }
}
