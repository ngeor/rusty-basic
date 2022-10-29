use crate::linter::arg_validation::ArgValidation;
use crate::parser::ExpressionNodes;
use rusty_common::QErrorNode;

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.is_empty() {
        Ok(())
    } else {
        args.require_one_numeric_argument()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn address_cannot_be_string() {
        let input = "DEF SEG = A$";
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }
}
