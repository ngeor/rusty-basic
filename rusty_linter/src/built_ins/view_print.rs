use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.is_empty() {
        Ok(())
    } else if args.len() == 2 {
        args.require_integer_argument(0)?;
        args.require_integer_argument(1)
    } else {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    }
}
