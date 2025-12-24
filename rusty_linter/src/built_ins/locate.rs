use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.len() < 2 || args.len() > 4 {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    } else {
        for i in 0..args.len() {
            args.require_integer_argument(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::core::LintError;

    #[test]
    fn lint_too_many_args() {
        assert_linter_err!("LOCATE 1, 2, 3, 4", LintError::ArgumentCountMismatch);
    }
}
