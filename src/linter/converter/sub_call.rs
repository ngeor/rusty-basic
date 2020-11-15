use crate::built_ins::BuiltInSub;
use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::converter::{ConverterImpl, ConverterWithImplicitVariables};
use crate::parser::{BareNameNode, ExpressionNodes, QualifiedNameNode, Statement, StatementNode};

impl<'a> ConverterImpl<'a> {
    pub fn sub_call(
        &mut self,
        sub_name_node: BareNameNode,
        args: ExpressionNodes,
    ) -> Result<(StatementNode, Vec<QualifiedNameNode>), QErrorNode> {
        let (converted_args, implicit_vars) = self.convert_and_collect_implicit_variables(args)?;
        let Locatable {
            element: sub_name,
            pos,
        } = sub_name_node;
        let opt_built_in: Option<BuiltInSub> = (&sub_name).into();
        match opt_built_in {
            Some(b) => Ok((
                Statement::BuiltInSubCall(b, converted_args).at(pos),
                implicit_vars,
            )),
            None => Ok((
                Statement::SubCall(sub_name, converted_args).at(pos),
                implicit_vars,
            )),
        }
    }
}
