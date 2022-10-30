use crate::linter::NameContext;
use rusty_common::{QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError};
use rusty_parser::{Expression, ExpressionNode, ExpressionNodes};

pub fn lint(args: &ExpressionNodes, name_context: NameContext) -> Result<(), QErrorNode> {
    if name_context == NameContext::Global {
        args.iter().try_for_each(require_constant)
    } else {
        Err(QError::IllegalInSubFunction).with_err_no_pos()
    }
}

fn require_constant(arg: &ExpressionNode) -> Result<(), QErrorNode> {
    match &arg.element {
        Expression::SingleLiteral(_)
        | Expression::DoubleLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::IntegerLiteral(_)
        | Expression::LongLiteral(_) => Ok(()),
        _ => Err(QError::InvalidConstant).with_err_at(arg),
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use rusty_common::*;

    #[test]
    fn data_not_allowed_in_sub() {
        let input = r#"
        SUB Hello
            DATA 1, 2
        END SUB
        "#;
        assert_linter_err!(input, QError::IllegalInSubFunction);
    }

    #[test]
    fn data_not_allowed_in_function() {
        let input = r#"
        FUNCTION Hello
            DATA 1, 2
        END FUNCTION
        "#;
        assert_linter_err!(input, QError::IllegalInSubFunction);
    }

    #[test]
    fn data_must_be_constants() {
        assert_linter_err!("DATA A", QError::InvalidConstant);
    }
}
