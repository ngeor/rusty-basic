
use crate::common::{CanCastTo, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
use crate::linter::arg_validation::ArgValidation;
use crate::parser::{ExpressionNodes, TypeQualifier};

pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
    if args.len() != 2 {
        Err(QError::ArgumentCountMismatch).with_err_no_pos()
    } else {
        args.require_integer_argument(0)?;
        if args[1].as_ref().can_cast_to(TypeQualifier::PercentInteger)
            || args[1].as_ref().can_cast_to(TypeQualifier::DollarString)
        {
            Ok(())
        } else {
            Err(QError::ArgumentTypeMismatch).with_err_at(&args[1])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::QError;

    #[test]
    fn string_without_args() {
        assert_linter_err!("PRINT STRING$()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn string_with_only_one_arg() {
        assert_linter_err!("PRINT STRING$(5)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn string_with_three_arguments() {
        assert_linter_err!("PRINT STRING$(1, 2, 3)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn string_with_string_first_argument() {
        assert_linter_err!(
            r#"PRINT STRING$("oops", "oops")"#,
            QError::ArgumentTypeMismatch
        );
    }
}
