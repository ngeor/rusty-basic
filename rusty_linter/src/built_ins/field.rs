use rusty_common::{AtPos, Position};
use rusty_parser::Expressions;

use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    // needs to be 1 + N*3 args, N >= 1
    // first is the file number
    // then the fields: width, variable name, variable
    if args.len() < 4 {
        return Err(LintError::ArgumentCountMismatch.at_pos(pos));
    }
    if (args.len() - 1) % 3 != 0 {
        return Err(LintError::ArgumentCountMismatch.at_pos(pos));
    }
    args.require_integer_argument(0)?;
    let mut i: usize = 1;
    while i < args.len() {
        args.require_integer_argument(i)?;
        args.require_string_argument(i + 1)?;
        args.require_string_variable(i + 2)?;
        i += 3;
    }
    Ok(())
}
