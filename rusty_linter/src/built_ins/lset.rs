use crate::arg_validation::ArgValidation;
use crate::error::{LintError, LintErrorPos};
use rusty_common::WithErrNoPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    // the parser should produce 3 arguments:
    // the variable name, as a string literal
    // the variable itself, a ByRef string variable
    // a string expression to assign to
    if args.len() != 3 {
        return Err(LintError::ArgumentCountMismatch).with_err_no_pos();
    }
    args.require_string_argument(0)?;
    // TODO ensure LSET is operating on variables previously used by FIELD in this scope
    args.require_string_variable(1)?;
    args.require_string_argument(2)
}
