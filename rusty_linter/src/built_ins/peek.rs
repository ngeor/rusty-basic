use crate::arg_validation::ArgValidation;
use rusty_common::QErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    args.require_one_numeric_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn must_have_arguments() {
        let input = "X = PEEK()";
        assert_linter_err!(input, QError::FunctionNeedsArguments);
    }

    #[test]
    fn two_arguments() {
        let input = "X = PEEK(1, 4)";
        assert_linter_err!(input, QError::ArgumentCountMismatch);
    }

    #[test]
    fn string_argument() {
        let input = "X = PEEK(A$)";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }
}
