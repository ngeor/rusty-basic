use super::built_in_sub_linter::is_built_in_sub;
use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::subprogram_context::SubMap;
use super::types::*;
use super::user_defined_function_linter::lint_call_args;
use crate::common::*;

pub struct UserDefinedSubLinter<'a> {
    pub subs: &'a SubMap,
}

impl<'a> PostConversionLinter for UserDefinedSubLinter<'a> {
    fn visit_sub_call(
        &self,
        name: &CaseInsensitiveString,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), Error> {
        if is_built_in_sub(name) {
            Ok(())
        } else {
            match self.subs.get(name) {
                Some((param_types, _)) => lint_call_args(args, param_types),
                None => err_no_pos(LinterError::SubprogramNotDefined),
            }
        }
    }
}
