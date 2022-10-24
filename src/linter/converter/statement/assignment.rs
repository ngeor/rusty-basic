use crate::common::*;
use crate::linter::converter::context::Context;
use crate::linter::converter::pos_context::PosContext;
use crate::linter::converter::traits::Convertible;
use crate::linter::converter::types::ExprContext;
use crate::parser::{Expression, ExpressionNode, Statement};

pub fn on_assignment(
    left: Expression,
    right: ExpressionNode,
    ctx: &mut PosContext,
) -> Result<Statement, QErrorNode> {
    assignment_pre_conversion_validation_rules::validate(ctx, &left)?;
    let converted_right: ExpressionNode = right.convert_in_default(ctx)?;
    let pos = ctx.pos();
    let Locatable {
        element: converted_left,
        ..
    } = left.at(pos).convert_in(ctx, ExprContext::Assignment)?;
    assignment_post_conversion_validation_rules::validate(&converted_left, &converted_right)?;
    Ok(Statement::Assignment(converted_left, converted_right))
}

mod assignment_pre_conversion_validation_rules {
    use super::*;
    use crate::parser::Expression;

    pub fn validate(ctx: &mut Context, left_side: &Expression) -> Result<(), QErrorNode> {
        cannot_assign_to_const(ctx, left_side)
    }

    fn cannot_assign_to_const(ctx: &mut Context, input: &Expression) -> Result<(), QErrorNode> {
        if let Expression::Variable(var_name, _) = input {
            if ctx.names.contains_const_recursively(var_name.bare_name()) {
                Err(QError::DuplicateDefinition).with_err_no_pos()
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

    pub fn validate(left_side: &Expression, right_side: &ExpressionNode) -> Result<(), QErrorNode> {
        if right_side.as_ref().can_cast_to(left_side) {
            Ok(())
        } else {
            Err(QError::TypeMismatch).with_err_at(right_side)
        }
    }
}
