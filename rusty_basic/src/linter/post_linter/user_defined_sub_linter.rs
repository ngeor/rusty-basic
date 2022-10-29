use super::post_conversion_linter::PostConversionLinter;
use super::user_defined_function_linter::lint_call_args;
use crate::linter::HasSubs;
use crate::parser::ExpressionNodes;
use rusty_common::*;

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
            Some(sub_signature_node) => {
                lint_call_args(args, sub_signature_node.as_ref().param_types())
            }
            None => Err(QError::SubprogramNotDefined).with_err_no_pos(),
        }
    }
}
