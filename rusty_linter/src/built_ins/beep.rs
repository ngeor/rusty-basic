use crate::built_ins::arg_validation::ArgValidation;
use crate::core::LintErrorPos;
use rusty_parser::specific::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    args.require_zero_arguments()
}
