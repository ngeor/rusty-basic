use crate::arg_validation::ArgValidation;
use rusty_common::QErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    args.require_one_double_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn no_args() {
        assert_linter_err!("PRINT MKD$()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn two_args() {
        assert_linter_err!("PRINT MKD$(A#, B#)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn first_arg_string() {
        assert_linter_err!("PRINT MKD$(\"10\")", QError::ArgumentTypeMismatch);
    }
}
