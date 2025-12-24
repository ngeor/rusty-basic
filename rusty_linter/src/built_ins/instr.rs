use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};
use rusty_common::AtPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.len() == 2 {
        args.require_string_argument(0)?;
        args.require_string_argument(1)
    } else if args.len() == 3 {
        args.require_integer_argument(0)?;
        args.require_string_argument(1)?;
        args.require_string_argument(2)
    } else {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::core::LintError;

    #[test]
    fn test_instr_linter() {
        assert_linter_err!(
            r#"PRINT INSTR("oops")"#,
            LintError::ArgumentCountMismatch,
            1,
            7
        );
    }
}
