use super::post_conversion_linter::PostConversionLinter;
use super::user_defined_function_linter::lint_call_args;
use crate::core::HasSubs;
use crate::core::{LintError, LintErrorPos};
use rusty_common::*;
use rusty_parser::specific::Expressions;

pub struct UserDefinedSubLinter<'a, R> {
    pub linter_context: &'a R,
}

impl<'a, R> PostConversionLinter for UserDefinedSubLinter<'a, R>
where
    R: HasSubs,
{
    fn visit_sub_call(
        &mut self,
        name: &CaseInsensitiveString,
        args: &Expressions,
    ) -> Result<(), LintErrorPos> {
        match self.linter_context.subs().get(name) {
            Some(sub_signature_pos) => {
                lint_call_args(args, sub_signature_pos.element.param_types())
            }
            None => Err(LintError::SubprogramNotDefined.at_no_pos()),
        }
    }
}
