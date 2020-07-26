use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::built_ins::{BuiltInLint, BuiltInSub};

pub struct BuiltInSubLinter;

impl PostConversionLinter for BuiltInSubLinter {
    fn visit_built_in_sub_call(
        &self,
        n: &BuiltInSub,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), Error> {
        n.lint(args)
    }
}
