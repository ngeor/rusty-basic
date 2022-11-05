use crate::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorPos, WithErrNoPos};
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    if args.len() != 2 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        args.require_string_argument(0)?;
        args.require_string_argument(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_name_linter_err() {
        assert_linter_err!(r#"NAME 1 AS "boo""#, QError::ArgumentTypeMismatch, 1, 6);
        assert_linter_err!(r#"NAME "boo" AS 2"#, QError::ArgumentTypeMismatch, 1, 15);
    }
}
