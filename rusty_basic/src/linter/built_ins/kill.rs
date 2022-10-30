use crate::linter::arg_validation::ArgValidation;
use rusty_common::QErrorNode;
use rusty_parser::ExpressionNodes;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    args.require_one_string_argument()
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_kill_linter() {
        assert_linter_err!("KILL", QError::ArgumentCountMismatch, 1, 1);
        assert_linter_err!(r#"KILL "a", "b""#, QError::ArgumentCountMismatch, 1, 1);
        assert_linter_err!(r#"KILL 42"#, QError::ArgumentTypeMismatch, 1, 6);
    }
}
