use crate::common::QErrorNode;
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::parser::{DimNameNode, QualifiedNameNode};

impl<'a> ConverterWithImplicitVariables<DimNameNode, DimNameNode> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        dim_name_node: DimNameNode,
    ) -> Result<(DimNameNode, Vec<QualifiedNameNode>), QErrorNode> {
        self.context.on_dim(dim_name_node)
    }
}
