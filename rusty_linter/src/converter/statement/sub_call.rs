use rusty_parser::{BuiltInSub, Statement, SubCall};

use crate::converter::common::{ConvertibleIn, ExprContext};
use crate::core::{LintErrorPos, LinterContext};

impl LinterContext {
    pub fn sub_call(&mut self, sub_call: SubCall) -> Result<Statement, LintErrorPos> {
        let (sub_name, args) = sub_call.into();
        let converted_args = args.convert_in(self, ExprContext::Argument)?;
        let opt_built_in: Option<BuiltInSub> = BuiltInSub::parse_non_keyword_sub(sub_name.as_ref());
        match opt_built_in {
            Some(b) => Ok(Statement::built_in_sub_call(b, converted_args)),
            None => Ok(Statement::sub_call(sub_name, converted_args)),
        }
    }
}
