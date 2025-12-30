use rusty_common::{AtPos, Position};
use rusty_parser::{Expressions, TypeQualifier};

use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{CanCastTo, LintError, LintErrorPos};

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    if args.len() != 2 {
        Err(LintError::ArgumentCountMismatch.at_pos(pos))
    } else {
        args.require_integer_argument(0)?;
        if args[1].can_cast_to(&TypeQualifier::PercentInteger)
            || args[1].can_cast_to(&TypeQualifier::DollarString)
        {
            Ok(())
        } else {
            Err(LintError::ArgumentTypeMismatch.at(&args[1]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_linter_err;

    #[test]
    fn string_without_args() {
        assert_linter_err!("PRINT STRING$()", LintError::FunctionNeedsArguments);
    }

    #[test]
    fn string_with_only_one_arg() {
        assert_linter_err!("PRINT STRING$(5)", LintError::ArgumentCountMismatch);
    }

    #[test]
    fn string_with_three_arguments() {
        assert_linter_err!("PRINT STRING$(1, 2, 3)", LintError::ArgumentCountMismatch);
    }

    #[test]
    fn string_with_string_first_argument() {
        assert_linter_err!(
            r#"PRINT STRING$("oops", "oops")"#,
            LintError::ArgumentTypeMismatch
        );
    }
}
