use crate::arg_validation::ArgValidation;
use crate::error::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.len() == 2 {
        args.require_string_argument(0)?;
        args.require_integer_argument(1)
    } else if args.len() == 3 {
        args.require_string_argument(0)?;
        args.require_integer_argument(1)?;
        args.require_integer_argument(2)
    } else {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn test_mid_linter() {
        assert_linter_err!(
            r#"PRINT MID$("oops")"#,
            LintError::ArgumentCountMismatch,
            1,
            7
        );
    }
}
