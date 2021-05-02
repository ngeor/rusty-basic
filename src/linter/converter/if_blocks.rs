use crate::linter::converter::conversion_traits::SameTypeConverterWithImplicits;
use crate::linter::converter::{ConverterImpl, ExprContext, R};
use crate::parser::{ConditionalBlockNode, IfBlockNode};

impl<'a> SameTypeConverterWithImplicits<ConditionalBlockNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(
        &mut self,
        a: ConditionalBlockNode,
    ) -> R<ConditionalBlockNode> {
        let (condition, mut implicit_vars) = self
            .context
            .on_expression(a.condition, ExprContext::Default)?;
        let (statements, mut block_implicits) =
            self.convert_block_keeping_implicits(a.statements)?;
        implicit_vars.append(&mut block_implicits);
        Ok((
            ConditionalBlockNode {
                condition,
                statements,
            },
            implicit_vars,
        ))
    }
}

impl<'a> SameTypeConverterWithImplicits<IfBlockNode> for ConverterImpl<'a> {
    fn convert_same_type_with_implicits(&mut self, a: IfBlockNode) -> R<IfBlockNode> {
        let (if_block, mut implicits) = self.convert_same_type_with_implicits(a.if_block)?;
        let (else_if_blocks, mut implicit_vars_else_if_blocks) =
            self.convert_same_type_with_implicits(a.else_if_blocks)?;
        implicits.append(&mut implicit_vars_else_if_blocks);
        let (else_block, mut implicits_else) =
            self.convert_same_type_with_implicits(a.else_block)?;
        implicits.append(&mut implicits_else);
        Ok((
            IfBlockNode {
                if_block,
                else_if_blocks,
                else_block,
            },
            implicits,
        ))
    }
}
