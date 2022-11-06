use crate::converter::context::Context;
use crate::converter::pos_context::PosContext;
use crate::converter::traits::Convertible;
use crate::converter::types::ExprContext;
use crate::error::LintErrorPos;
use rusty_common::{AtPos, HasPos, Positioned};
use rusty_parser::{Expression, ExpressionPos, Statement};

pub fn on_assignment(
    left: Expression,
    right: ExpressionPos,
    ctx: &mut PosContext,
) -> Result<Statement, LintErrorPos> {
    assignment_pre_conversion_validation_rules::validate(ctx, &left)?;
    let converted_right: ExpressionPos = right.convert_in_default(ctx)?;
    let pos = ctx.pos();
    let Positioned {
        element: converted_left,
        ..
    } = left.at_pos(pos).convert_in(ctx, ExprContext::Assignment)?;
    assignment_post_conversion_validation_rules::validate(&converted_left, &converted_right)?;
    Ok(Statement::Assignment(converted_left, converted_right))
}

mod assignment_pre_conversion_validation_rules {
    use super::*;
    use crate::LintError;
    use rusty_common::WithErrNoPos;

    pub fn validate(ctx: &mut Context, left_side: &Expression) -> Result<(), LintErrorPos> {
        cannot_assign_to_const(ctx, left_side)
    }

    fn cannot_assign_to_const(ctx: &mut Context, input: &Expression) -> Result<(), LintErrorPos> {
        if let Expression::Variable(var_name, _) = input {
            if ctx.names.contains_const_recursively(var_name.bare_name()) {
                Err(LintError::DuplicateDefinition).with_err_no_pos()
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
    use crate::CanCastTo;
    use crate::LintError;
    use rusty_common::WithErrAt;

    pub fn validate(
        left_side: &Expression,
        right_side: &ExpressionPos,
    ) -> Result<(), LintErrorPos> {
        if right_side.can_cast_to(left_side) {
            Ok(())
        } else {
            Err(LintError::TypeMismatch).with_err_at(right_side)
        }
    }
}
