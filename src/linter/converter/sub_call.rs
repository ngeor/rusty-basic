use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::converter::Context;
use crate::linter::converter::expr_rules::ExprContext;
use crate::linter::converter::traits::Convertible;
use crate::parser::{BareNameNode, ExpressionNodes, Statement, StatementNode};

impl Context {
    pub fn sub_call(
        &mut self,
        sub_name_node: BareNameNode,
        args: ExpressionNodes,
    ) -> Result<StatementNode, QErrorNode> {
        let converted_args = args.convert_in(self, ExprContext::Argument)?;
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
