use crate::common::QErrorNode;
use crate::linter::converter::context::Context;
use crate::linter::converter::traits::Convertible;
use crate::linter::converter::types::ExprContext;
use crate::parser::{BareName, BuiltInSub, ExpressionNodes, Statement};

impl Context {
    pub fn sub_call(
        &mut self,
        sub_name: BareName,
        args: ExpressionNodes,
    ) -> Result<Statement, QErrorNode> {
        let converted_args = args.convert_in(self, ExprContext::Argument)?;
        let opt_built_in: Option<BuiltInSub> = BuiltInSub::parse_non_keyword_sub(sub_name.as_ref());
        match opt_built_in {
            Some(b) => Ok(Statement::BuiltInSubCall(b, converted_args)),
            None => Ok(Statement::SubCall(sub_name, converted_args)),
        }
    }
}
