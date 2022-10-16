use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::{ConverterImpl, ExprContext};
use crate::parser::{BareNameNode, ExpressionNodes, Statement, StatementNode};

impl ConverterImpl {
    pub fn sub_call(
        &mut self,
        sub_name_node: BareNameNode,
        args: ExpressionNodes,
    ) -> Result<StatementNode, QErrorNode> {
        let converted_args = self.context.on_expressions(args, ExprContext::Parameter)?;
        let Locatable {
            element: sub_name,
            pos,
        } = sub_name_node;
        let opt_built_in: Option<BuiltInSub> = BuiltInSub::parse_non_keyword_sub(sub_name.as_ref());
        match opt_built_in {
            Some(b) => Ok(Statement::BuiltInSubCall(b, converted_args).at(pos)),
            None => Ok(Statement::SubCall(sub_name, converted_args).at(pos)),
        }
    }
}
