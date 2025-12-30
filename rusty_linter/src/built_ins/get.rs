use rusty_common::{AtPos, Position};
use rusty_parser::Expressions;

use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    if args.len() != 2 {
        return Err(LintError::ArgumentCountMismatch.at_pos(pos));
    }
    args.require_integer_argument(0)?;
    args.require_long_argument(1)
}
