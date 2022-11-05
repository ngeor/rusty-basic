use crate::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorPos, WithErrNoPos};
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    if args.len() != 2 {
        return Err(QError::ArgumentCountMismatch).with_err_no_pos();
    }
    args.require_integer_argument(0)?;
    args.require_long_argument(1)
}
