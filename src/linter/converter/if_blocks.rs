use crate::common::QErrorNode;
use crate::linter::converter::converter::{
    Converter, ConverterImpl, ConverterWithImplicitVariables,
};
use crate::linter::{ConditionalBlockNode, IfBlockNode};
use crate::parser;
use crate::parser::QualifiedNameNode;

impl<'a> ConverterWithImplicitVariables<parser::ConditionalBlockNode, ConditionalBlockNode>
    for ConverterImpl<'a>
{
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: parser::ConditionalBlockNode,
    ) -> Result<(ConditionalBlockNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (condition, implicit_vars) =
            self.convert_and_collect_implicit_variables(a.condition)?;
        Ok((
            ConditionalBlockNode {
                condition,
                statements: self.convert(a.statements)?,
            },
            implicit_vars,
        ))
    }
}

impl<'a> ConverterWithImplicitVariables<parser::IfBlockNode, IfBlockNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        a: parser::IfBlockNode,
    ) -> Result<(IfBlockNode, Vec<QualifiedNameNode>), QErrorNode> {
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
