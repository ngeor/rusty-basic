use rusty_common::Position;
use rusty_parser::Expressions;

use crate::built_ins::arg_validation::ArgValidation;
use crate::core::LintErrorPos;

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    args.require_one_numeric_argument(pos)
}
