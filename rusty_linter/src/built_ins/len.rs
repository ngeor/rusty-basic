use crate::CanCastTo;
use rusty_common::{QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
use rusty_parser::{
    ExpressionNode, ExpressionNodes, ExpressionTrait, ExpressionType, HasExpressionType,
    TypeQualifier,
};

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() != 1 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        let arg: &ExpressionNode = &args[0];
        if arg.is_by_ref() {
            match arg.expression_type() {
                // QBasic actually accepts LEN(A) where A is an array,
                // but its results don't make much sense
                ExpressionType::Unresolved | ExpressionType::Array(_) => {
                    Err(QError::ArgumentTypeMismatch).with_err_at(arg)
                }
                _ => Ok(()),
            }
        } else if !arg.can_cast_to(&TypeQualifier::DollarString) {
            Err(QError::VariableRequired).with_err_at(arg)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn test_len_integer_expression_error() {
        let program = "PRINT LEN(42)";
        assert_linter_err!(program, QError::VariableRequired, 1, 11);
    }

    #[test]
    fn test_len_integer_const_error() {
        let program = "
            CONST X = 42
            PRINT LEN(X)
            ";
        assert_linter_err!(program, QError::VariableRequired, 3, 23);
    }

    #[test]
    fn test_len_two_arguments_error() {
        let program = r#"PRINT LEN("a", "b")"#;
        assert_linter_err!(program, QError::ArgumentCountMismatch, 1, 7);
    }

    #[test]
    fn test_array() {
        let program = r#"
        DIM A(1 TO 2) AS INTEGER
        PRINT LEN(A)
        "#;
        // QBasic actually seems to be printing "4" regardless of the array type
        assert_linter_err!(program, QError::ArgumentTypeMismatch);
    }
}
