use rusty_common::{AtPos, Position};
use rusty_parser::Expressions;

use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    if args.len() == 2 {
        args.require_string_argument(0)?;
        args.require_integer_argument(1)
    } else if args.len() == 3 {
        args.require_string_argument(0)?;
        args.require_integer_argument(1)?;
        args.require_integer_argument(2)
    } else {
        Err(LintError::ArgumentCountMismatch.at_pos(pos))
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::core::LintError;

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
