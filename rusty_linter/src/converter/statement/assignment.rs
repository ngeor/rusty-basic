use crate::converter::common::Context;
use crate::converter::common::ConvertibleIn;
use crate::converter::common::ExprContext;
use crate::core::LintErrorPos;
use rusty_common::Position;
use rusty_common::{AtPos, Positioned};
use rusty_parser::Assignment;
use rusty_parser::{Expression, ExpressionPos, Statement};

pub fn on_assignment(
    a: Assignment,
    ctx: &mut Context,
    pos: Position,
) -> Result<Statement, LintErrorPos> {
    let (left, right) = a.into();
    assignment_pre_conversion_validation_rules::validate(ctx, &left, pos)?;
    let converted_right: ExpressionPos = right.convert_in_default(ctx)?;
    let Positioned {
        element: converted_left,
        ..
    } = left.at_pos(pos).convert_in(ctx, ExprContext::Assignment)?;
    assignment_post_conversion_validation_rules::validate(&converted_left, &converted_right)?;
    Ok(Statement::assignment(converted_left, converted_right))
}

mod assignment_pre_conversion_validation_rules {
    use super::*;
    use crate::core::LintError;

    pub fn validate(
        ctx: &mut Context,
        left_side: &Expression,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        cannot_assign_to_const(ctx, left_side, pos)
    }

    fn cannot_assign_to_const(
        ctx: &mut Context,
        input: &Expression,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        if let Expression::Variable(var_name, _) = input {
            if ctx.names.contains_const_recursively(var_name.bare_name()) {
                Err(LintError::DuplicateDefinition.at_pos(pos))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

mod assignment_post_conversion_validation_rules {
    use super::*;
    use crate::core::CanCastTo;
    use crate::core::LintError;

    pub fn validate(
        left_side: &Expression,
        right_side: &ExpressionPos,
    ) -> Result<(), LintErrorPos> {
        if right_side.can_cast_to(left_side) {
            Ok(())
        } else {
            Err(LintError::TypeMismatch.at(right_side))
        }
    }
}
