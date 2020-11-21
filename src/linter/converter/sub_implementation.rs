use crate::common::QErrorNode;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::parser::{
    BareNameNode, ParamNameNodes, StatementNodes, SubImplementation, TopLevelToken,
};

impl<'a> ConverterImpl<'a> {
    pub fn convert_sub_implementation(
        &mut self,
        sub_name_node: BareNameNode,
        params: ParamNameNodes,
        block: StatementNodes,
    ) -> Result<TopLevelToken, QErrorNode> {
        let mapped_params = self.context.push_sub_context(params)?;
        let mapped = TopLevelToken::SubImplementation(SubImplementation {
            name: sub_name_node,
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.context.pop_context();
        Ok(mapped)
    }
}
