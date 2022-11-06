use crate::arg_validation::ArgValidation;
use crate::error::{LintError, LintErrorPos};
use rusty_common::WithErrNoPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.len() != 2 {
        return Err(LintError::ArgumentCountMismatch).with_err_no_pos();
    }
    args.require_integer_argument(0)?;
    args.require_long_argument(1)
}
