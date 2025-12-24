use crate::core::LintErrorPos;
use rusty_parser::specific::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    super::get::lint(args)
}
