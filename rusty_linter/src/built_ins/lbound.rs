use rusty_common::{AtPos, Position, Positioned};
use rusty_parser::{Expression, ExpressionType, Expressions, TypeQualifier};

use crate::core::{CanCastTo, LintError, LintErrorPos};

pub fn lint(args: &Expressions, pos: Position) -> Result<(), LintErrorPos> {
    if args.is_empty() || args.len() > 2 {
        return Err(LintError::ArgumentCountMismatch.at_pos(pos));
    }

    // Can have at one or two arguments. First must be the array name, without parenthesis.
    // Second, optional, is an integer specifying the array dimension >=1 (default is 1).
    let Positioned {
        element: first,
        pos: first_pos,
    } = args.first().unwrap();
    if let Expression::Variable(_, ExpressionType::Array(_)) = first {
        if args.len() == 2 {
            if args[1].can_cast_to(&TypeQualifier::PercentInteger) {
                Ok(())
            } else {
                Err(LintError::ArgumentTypeMismatch.at(&args[1]))
            }
        } else {
            Ok(())
        }
    } else {
        Err(LintError::ArgumentTypeMismatch.at(first_pos))
    }
}
