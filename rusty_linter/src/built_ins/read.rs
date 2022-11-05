use crate::arg_validation::ArgValidation;
use rusty_common::{QError, QErrorPos, WithErrNoPos};
use rusty_parser::Expressions;

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    if args.is_empty() {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        for i in 0..args.len() {
            args.require_variable_of_built_in_type(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn single_literal_argument_not_allowed() {
        assert_linter_err!("READ 3.14", QError::VariableRequired);
    }

    #[test]
    fn double_literal_argument_not_allowed() {
        assert_linter_err!("READ 3.14#", QError::VariableRequired);
    }

    #[test]
    fn string_literal_argument_not_allowed() {
        assert_linter_err!("READ \"hello\"", QError::VariableRequired);
    }

    #[test]
    fn integer_literal_argument_not_allowed() {
        assert_linter_err!("READ 42", QError::VariableRequired);
    }

    #[test]
    fn long_literal_argument_not_allowed() {
        assert_linter_err!("READ 65536", QError::VariableRequired);
    }

    #[test]
    fn function_call_argument_not_allowed() {
        let input = r#"
        READ Hello(1)
        FUNCTION Hello(A)
        END FUNCTION
        "#;
        assert_linter_err!(input, QError::VariableRequired);
    }

    #[test]
    fn built_in_function_call_argument_not_allowed() {
        assert_linter_err!("READ LEN(A)", QError::VariableRequired);
    }

    #[test]
    fn binary_expression_argument_not_allowed() {
        assert_linter_err!("READ A + B", QError::VariableRequired);
    }

    #[test]
    fn unary_expression_argument_not_allowed() {
        assert_linter_err!("READ NOT A", QError::VariableRequired);
    }

    #[test]
    fn parenthesis_expression_argument_not_allowed() {
        assert_linter_err!("READ (A)", QError::VariableRequired);
    }

    #[test]
    fn array_variable_argument_not_allowed() {
        let input = r#"
        DIM A(1 TO 5)
        READ A
        "#;
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
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
        assert_linter_err!(input, QError::ArgumentTypeMismatch);
    }
}
