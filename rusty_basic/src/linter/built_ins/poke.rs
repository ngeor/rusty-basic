use crate::linter::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() != 2 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
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
    use rusty_common::*;

    #[test]
    fn must_have_arguments() {
        let input = "POKE";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }

    #[test]
    fn one_argument() {
        let input = "POKE 42";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }

    #[test]
    fn three_arguments() {
        let input = "POKE 1, 2, 3";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }

    #[test]
    fn string_first_argument() {
        let input = "POKE A$, 1";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn string_second_argument() {
        let input = "POKE 1, A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }
}
