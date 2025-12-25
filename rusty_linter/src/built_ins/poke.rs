use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};
use rusty_common::{AtPos, Position};
use rusty_parser::Expressions;

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    if args.len() != 2 {
        Err(LintError::ArgumentCountMismatch.at_pos(pos))
    } else {
        for i in 0..args.len() {
            args.require_integer_argument(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_linter_err;

    #[test]
    fn must_have_arguments() {
        let input = "POKE";
        assert_linter_err!(input, LintError::ArgumentCountMismatch);
    }

    #[test]
    fn one_argument() {
        let input = "POKE 42";
        assert_linter_err!(input, LintError::ArgumentCountMismatch);
    }

    #[test]
    fn three_arguments() {
        let input = "POKE 1, 2, 3";
        assert_linter_err!(input, LintError::ArgumentCountMismatch);
    }

    #[test]
    fn string_first_argument() {
        let input = "POKE A$, 1";
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }

    #[test]
    fn string_second_argument() {
        let input = "POKE 1, A$";
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }
}
