use crate::linter::converter::context::ExprContext;
use crate::linter::converter::{Converter, ConverterImpl, ConverterWithImplicitVariables, R};
use crate::parser::{ConditionalBlockNode, IfBlockNode};

impl<'a> ConverterWithImplicitVariables<ConditionalBlockNode, ConditionalBlockNode>
    for ConverterImpl<'a>
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: ConditionalBlockNode,
    ) -> R<ConditionalBlockNode> {
        let (condition, implicit_vars) = self
            .context
            .on_expression(a.condition, ExprContext::Default)?;
        Ok((
            ConditionalBlockNode {
                condition,
                statements: self.convert(a.statements)?,
            },
            implicit_vars,
        ))
    }
}

impl<'a> ConverterWithImplicitVariables<IfBlockNode, IfBlockNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(&mut self, a: IfBlockNode) -> R<IfBlockNode> {
        let (if_block, mut implicit_vars_if_block) =
            self.convert_and_collect_implicit_variables(a.if_block)?;
        let (else_if_blocks, mut implicit_vars_else_if_blocks) =
            self.convert_and_collect_implicit_variables(a.else_if_blocks)?;

        implicit_vars_if_block.append(&mut implicit_vars_else_if_blocks);

        Ok((
            IfBlockNode {
                if_block,
                else_if_blocks,
                else_block: self.convert(a.else_block)?,
            },
            implicit_vars_if_block,
        ))
    }
}
