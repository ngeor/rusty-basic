use crate::built_ins::arg_validation::ArgValidation;
use crate::core::{LintError, LintErrorPos};
use rusty_common::{AtPos, Position};
use rusty_parser::Expressions;

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    if args.is_empty() {
        Err(LintError::ArgumentCountMismatch.at_pos(pos))
    } else {
        for i in 0..args.len() {
            args.require_variable_of_built_in_type(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_linter_err;

    #[test]
    fn single_literal_argument_not_allowed() {
        assert_linter_err!("READ 3.14", LintError::VariableRequired);
    }

    #[test]
    fn double_literal_argument_not_allowed() {
        assert_linter_err!("READ 3.14#", LintError::VariableRequired);
    }

    #[test]
    fn string_literal_argument_not_allowed() {
        assert_linter_err!("READ \"hello\"", LintError::VariableRequired);
    }

    #[test]
    fn integer_literal_argument_not_allowed() {
        assert_linter_err!("READ 42", LintError::VariableRequired);
    }

    #[test]
    fn long_literal_argument_not_allowed() {
        assert_linter_err!("READ 65536", LintError::VariableRequired);
    }

    #[test]
    fn function_call_argument_not_allowed() {
        let input = r#"
        READ Hello(1)
        FUNCTION Hello(A)
        END FUNCTION
        "#;
        assert_linter_err!(input, LintError::VariableRequired);
    }

    #[test]
    fn built_in_function_call_argument_not_allowed() {
        assert_linter_err!("READ LEN(A)", LintError::VariableRequired);
    }

    #[test]
    fn binary_expression_argument_not_allowed() {
        assert_linter_err!("READ A + B", LintError::VariableRequired);
    }

    #[test]
    fn unary_expression_argument_not_allowed() {
        assert_linter_err!("READ NOT A", LintError::VariableRequired);
    }

    #[test]
    fn parenthesis_expression_argument_not_allowed() {
        assert_linter_err!("READ (A)", LintError::VariableRequired);
    }

    #[test]
    fn array_variable_argument_not_allowed() {
        let input = r#"
        DIM A(1 TO 5)
        READ A
        "#;
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }

    #[test]
    fn user_defined_type_argument_not_allowed() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
        END TYPE
        DIM C AS Card
        READ C
        "#;
        assert_linter_err!(input, LintError::ArgumentTypeMismatch);
    }
}
