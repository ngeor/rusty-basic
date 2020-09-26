use crate::built_ins::BuiltInSub;
use crate::common::QErrorNode;
use crate::linter::converter::converter::{Converter, ConverterImpl};
use crate::linter::Statement;
use crate::parser::{BareName, ExpressionNodes};

impl<'a> ConverterImpl<'a> {
    pub fn sub_call(
        &mut self,
        sub_name: BareName,
        args: ExpressionNodes,
    ) -> Result<Statement, QErrorNode> {
        let converted_args = self.convert(args)?;
        let opt_built_in: Option<BuiltInSub> = (&sub_name).into();
        match opt_built_in {
            Some(b) => Ok(Statement::BuiltInSubCall(b, converted_args)),
            None => Ok(Statement::SubCall(sub_name, converted_args)),
        }
    }
}
