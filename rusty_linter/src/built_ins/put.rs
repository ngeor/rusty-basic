use rusty_common::Position;
use rusty_parser::Expressions;

use crate::core::LintErrorPos;

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    super::get::lint(args, pos)
}
