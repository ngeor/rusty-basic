use rusty_parser::Expressions;

use crate::built_ins::arg_validation::ArgValidation;
use crate::core::LintErrorPos;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    for i in 0..args.len() {
        args.require_integer_argument(i)?;
    }
    Ok(())
}
