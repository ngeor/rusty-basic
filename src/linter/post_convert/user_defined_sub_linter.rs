use super::post_conversion_linter::PostConversionLinter;
use super::user_defined_function_linter::lint_call_args;
use crate::common::*;
use crate::linter::pre_convert::subprogram_context::SubMap;
use crate::linter::types::*;

pub struct UserDefinedSubLinter<'a> {
    pub subs: &'a SubMap,
}

impl<'a> PostConversionLinter for UserDefinedSubLinter<'a> {
    fn visit_sub_call(
        &self,
        name: &CaseInsensitiveString,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), QErrorNode> {
        match self.subs.get(name) {
            Some(Locatable {
                element: param_types,
                ..
            }) => lint_call_args(args, param_types),
            None => err_no_pos(QError::SubprogramNotDefined),
        }
    }
}
