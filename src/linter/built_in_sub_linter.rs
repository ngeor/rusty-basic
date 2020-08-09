use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::built_ins::{BuiltInLint, BuiltInSub};

/// Lints built-in subs. Delegates responsibility to the built-in subs
/// themselves in the built_ins module.
pub struct BuiltInSubLinter;

impl PostConversionLinter for BuiltInSubLinter {
    fn visit_built_in_sub_call(
        &self,
        built_in_sub: &BuiltInSub,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), LinterErrorNode> {
        built_in_sub.lint(args)
    }
}
