use crate::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorPos, WithErrNoPos};
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    // 1 or 2 arguments (row, col) but they're duplicated because they're optional
    if args.len() > 4 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        for i in 0..args.len() {
            args.require_integer_argument(i)?;
        }
        Ok(())
    }
}
