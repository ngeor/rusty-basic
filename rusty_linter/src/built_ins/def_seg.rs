use crate::arg_validation::ArgValidation;
use rusty_common::QErrorPos;
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
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
