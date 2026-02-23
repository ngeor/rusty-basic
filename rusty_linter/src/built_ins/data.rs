use rusty_common::{AtPos, Position};
use rusty_parser::{Expression, ExpressionPos, Expressions};

use crate::core::{LintError, LintErrorPos, NameScope};

pub fn lint(args: &Expressions, name_scope: NameScope, pos: Position) -> Result<(), LintErrorPos> {
    if name_scope == NameScope::Global {
        args.iter().try_for_each(require_constant)
    } else {
        Err(LintError::IllegalInSubFunction.at_pos(pos))
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
