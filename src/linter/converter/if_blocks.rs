use crate::common::QErrorNode;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::{ConditionalBlockNode, IfBlockNode};
use crate::parser;

impl<'a> Converter<parser::ConditionalBlockNode, ConditionalBlockNode> for ConverterImpl<'a> {
    fn convert(
        &mut self,
        a: parser::ConditionalBlockNode,
    ) -> Result<ConditionalBlockNode, QErrorNode> {
        Ok(ConditionalBlockNode {
            condition: self.convert(a.condition)?,
            statements: self.convert(a.statements)?,
        })
    }
}

impl<'a> Converter<parser::IfBlockNode, IfBlockNode> for ConverterImpl<'a> {
    fn convert(&mut self, a: parser::IfBlockNode) -> Result<IfBlockNode, QErrorNode> {
        Ok(IfBlockNode {
            if_block: self.convert(a.if_block)?,
            else_if_blocks: self.convert(a.else_if_blocks)?,
            else_block: self.convert(a.else_block)?,
        })
    }
}
