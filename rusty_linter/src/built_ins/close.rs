use crate::arg_validation::ArgValidation;
use crate::error::LintErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    for i in 0..args.len() {
        args.require_integer_argument(i)?;
    }
    Ok(())
}
