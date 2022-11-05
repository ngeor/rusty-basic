use rusty_common::QErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    super::lbound::lint(args)
}
