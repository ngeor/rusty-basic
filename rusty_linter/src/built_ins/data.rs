use crate::error::{LintError, LintErrorPos};
use crate::NameContext;
use rusty_common::AtPos;
use rusty_parser::{Expression, ExpressionPos, Expressions};

pub fn lint(args: &Expressions, name_context: NameContext) -> Result<(), LintErrorPos> {
    if name_context == NameContext::Global {
        args.iter().try_for_each(require_constant)
    } else {
        Err(LintError::IllegalInSubFunction.at_no_pos())
    }
}

fn require_constant(arg: &ExpressionPos) -> Result<(), LintErrorPos> {
    match &arg.element {
        Expression::SingleLiteral(_)
        | Expression::DoubleLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::IntegerLiteral(_)
        | Expression::LongLiteral(_) => Ok(()),
        _ => Err(LintError::InvalidConstant.at(arg)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_linter_err;

    #[test]
    fn data_not_allowed_in_sub() {
        let input = r#"
        SUB Hello
            DATA 1, 2
        END SUB
        "#;
        assert_linter_err!(input, LintError::IllegalInSubFunction);
    }

    #[test]
    fn data_not_allowed_in_function() {
        let input = r#"
        FUNCTION Hello
            DATA 1, 2
        END FUNCTION
        "#;
        assert_linter_err!(input, LintError::IllegalInSubFunction);
    }

    #[test]
    fn data_must_be_constants() {
        assert_linter_err!("DATA A", LintError::InvalidConstant);
    }
}
