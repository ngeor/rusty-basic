use crate::arg_validation::ArgValidation;
use crate::error::{LintError, LintErrorPos};
use rusty_common::*;
use rusty_parser::{Expression, Expressions};

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    // the first one or two arguments stand for the file number
    // if the first argument is 0, no file handle
    // if the first argument is 1, the second is the file handle

    if args.len() <= 1 {
        return Err(LintError::ArgumentCountMismatch.at_no_pos());
    }
    let mut has_file_number: bool = false;
    if let Positioned {
        element: Expression::IntegerLiteral(0),
        ..
    } = args[0]
    {
        // does not have a file number
    } else if let Positioned {
        element: Expression::IntegerLiteral(1),
        ..
    } = args[0]
    {
        // must have a file number
        args.require_integer_argument(1)?;
        has_file_number = true;
    } else {
        panic!("parser sent unexpected arguments");
    }

    let starting_index = if has_file_number { 2 } else { 1 };
    if args.len() <= starting_index {
        return Err(LintError::ArgumentCountMismatch.at_no_pos());
    }

    for i in starting_index..args.len() {
        args.require_variable_of_built_in_type(i)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_linter_err;

    #[test]
    fn test_parenthesis_variable_required() {
        let input = "INPUT (A$)";
        assert_linter_err!(input, LintError::VariableRequired);
    }

    #[test]
    fn test_binary_expression_variable_required() {
        let input = "INPUT A$ + B$";
        assert_linter_err!(input, LintError::VariableRequired);
    }

    #[test]
    fn test_const() {
        let input = r#"
                CONST A$ = "hello"
                INPUT A$
                "#;
        assert_linter_err!(input, LintError::VariableRequired);
    }
}
