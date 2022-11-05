use crate::CanCastTo;
use rusty_common::{Positioned, QError, QErrorPos, WithErrAt, WithErrNoPos};
use rusty_parser::{Expression, ExpressionType, Expressions, TypeQualifier, VariableInfo};

pub fn lint(args: &Expressions) -> Result<(), QErrorPos> {
    if args.is_empty() || args.len() > 2 {
        return Err(QError::ArgumentCountMismatch).with_err_no_pos();
    }

    // Can have at one or two arguments. First must be the array name, without parenthesis.
    // Second, optional, is an integer specifying the array dimension >=1 (default is 1).
    let Positioned {
        element: first,
        pos: first_pos,
    } = args.get(0).unwrap();
    if let Expression::Variable(
        _,
        VariableInfo {
            expression_type: ExpressionType::Array(_),
            ..
        },
    ) = first
    {
        if args.len() == 2 {
            if args[1].can_cast_to(&TypeQualifier::PercentInteger) {
                Ok(())
            } else {
                Err(QError::ArgumentTypeMismatch).with_err_at(&args[1])
            }
        } else {
            Ok(())
        }
    } else {
        Err(QError::ArgumentTypeMismatch).with_err_at(first_pos)
    }
}
