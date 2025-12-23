use crate::error::{LintError, LintErrorPos};
use crate::CanCastTo;
use rusty_common::AtPos;
use rusty_parser::specific::{
    ExpressionPos, ExpressionTrait, ExpressionType, Expressions, HasExpressionType, TypeQualifier,
};

pub fn lint(args: &Expressions) -> Result<(), LintErrorPos> {
    if args.len() != 1 {
        Err(LintError::ArgumentCountMismatch.at_no_pos())
    } else {
        let arg: &ExpressionPos = &args[0];
        if arg.is_by_ref() {
            match arg.expression_type() {
                // QBasic actually accepts LEN(A) where A is an array,
                // but its results don't make much sense
                ExpressionType::Unresolved | ExpressionType::Array(_) => {
                    Err(LintError::ArgumentTypeMismatch.at(arg))
                }
                _ => Ok(()),
            }
        } else if !arg.can_cast_to(&TypeQualifier::DollarString) {
            Err(LintError::VariableRequired.at(arg))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::LintError;

    #[test]
    fn test_len_integer_expression_error() {
        let program = "PRINT LEN(42)";
        assert_linter_err!(program, LintError::VariableRequired, 1, 11);
    }

    #[test]
    fn test_len_integer_const_error() {
        let program = "
            CONST X = 42
            PRINT LEN(X)
            ";
        assert_linter_err!(program, LintError::VariableRequired, 3, 23);
    }

    #[test]
    fn test_len_two_arguments_error() {
        let program = r#"PRINT LEN("a", "b")"#;
        assert_linter_err!(program, LintError::ArgumentCountMismatch, 1, 7);
    }

    #[test]
    fn test_array() {
        let program = r#"
        DIM A(1 TO 2) AS INTEGER
        PRINT LEN(A)
        "#;
        // QBasic actually seems to be printing "4" regardless of the array type
        assert_linter_err!(program, LintError::ArgumentTypeMismatch);
    }
}
