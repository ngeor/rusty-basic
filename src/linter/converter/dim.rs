use crate::common::QErrorNode;
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::parser::{DimList, QualifiedNameNode};

impl<'a> ConverterWithImplicitVariables<DimList, DimList> for ConverterImpl<'a> {
    fn convert_and_collect_implicit_variables(
        &mut self,
        dim_list: DimList,
    ) -> Result<(DimList, Vec<QualifiedNameNode>), QErrorNode> {
        self.context.on_dim(dim_list)
    }
}
