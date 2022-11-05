use crate::arg_validation::ArgValidation;
use rusty_common::QErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    for i in 0..args.len() {
        args.require_integer_argument(i)?;
    }
    Ok(())
}
