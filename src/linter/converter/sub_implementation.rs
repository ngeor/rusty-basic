use crate::common::QErrorNode;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::parser::{
    BareName, BareNameNode, ParamNameNodes, StatementNodes, SubImplementation, TopLevelToken,
};

impl<'a> ConverterImpl<'a> {
    pub fn convert_sub_implementation(
        &mut self,
        sub_name_node: BareNameNode,
        params: ParamNameNodes,
        block: StatementNodes,
    ) -> Result<TopLevelToken, QErrorNode> {
        let sub_name: &BareName = sub_name_node.as_ref();
        self.push_sub_context(sub_name.clone());
        let mapped_params = self.resolve_params(params, None)?;
        let mapped = TopLevelToken::SubImplementation(SubImplementation {
            name: sub_name_node,
            params: mapped_params,
            body: self.convert(block)?,
        });
        self.pop_context();
        Ok(mapped)
    }
}
