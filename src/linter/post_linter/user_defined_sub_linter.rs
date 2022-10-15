use super::post_conversion_linter::PostConversionLinter;
use super::user_defined_function_linter::lint_call_args;
use crate::common::*;
use crate::linter::pre_linter::HasSubs;
use crate::parser::ExpressionNodes;

pub struct UserDefinedSubLinter<'a, R> {
    pub context: &'a R,
}

impl<'a, R> PostConversionLinter for UserDefinedSubLinter<'a, R>
where
    R: HasSubs,
{
    fn visit_sub_call(
        &mut self,
        name: &CaseInsensitiveString,
        args: &ExpressionNodes,
    ) -> Result<(), QErrorNode> {
        match self.context.subs().get(name) {
            Some(Locatable {
                element: param_types,
                ..
            }) => lint_call_args(args, param_types),
            None => err_no_pos(QError::SubprogramNotDefined),
        }
    }
}
