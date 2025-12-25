use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};
use rusty_common::{AtPos, Position};
use rusty_parser::Expressions;

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    // 1 or 2 arguments (row, col) but they're duplicated because they're optional
    if args.len() > 4 {
        Err(LintError::ArgumentCountMismatch.at_pos(pos))
    } else {
        for i in 0..args.len() {
            args.require_integer_argument(i)?;
        }
        Ok(())
    }
}
