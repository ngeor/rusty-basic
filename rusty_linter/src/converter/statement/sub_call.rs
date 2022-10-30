use crate::converter::context::Context;
use crate::converter::traits::Convertible;
use crate::converter::types::ExprContext;
use rusty_common::QErrorNode;
use rusty_parser::{BareName, BuiltInSub, ExpressionNodes, Statement};

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
