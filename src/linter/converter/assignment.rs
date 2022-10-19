use crate::common::*;
use crate::linter::converter::{Context, ExprContext};
use crate::parser::{ExpressionNode, Statement, StatementNode};

impl Context {
    pub fn assignment(
        &mut self,
        name_expr_node: ExpressionNode,
        expression_node: ExpressionNode,
    ) -> Result<StatementNode, QErrorNode> {
        self.on_assignment(name_expr_node, expression_node).map(
            |(Locatable { element: left, pos }, right)| Statement::Assignment(left, right).at(pos),
        )
    }
}

pub fn on_assignment(
    context: &mut Context,
    left_side: ExpressionNode,
    right_side: ExpressionNode,
) -> Result<(ExpressionNode, ExpressionNode), QErrorNode> {
    assignment_pre_conversion_validation_rules::validate(context, &left_side)?;
    let converted_right_side = context.on_expression(right_side, ExprContext::Default)?;
    let converted_left_side = context.on_expression(left_side, ExprContext::Assignment)?;
    assignment_post_conversion_validation_rules::validate(
        &converted_left_side,
        &converted_right_side,
    )?;
    Ok((converted_left_side, converted_right_side))
}

mod assignment_pre_conversion_validation_rules {
    use super::*;
    use crate::parser::Expression;

    pub fn validate(ctx: &mut Context, left_side: &ExpressionNode) -> Result<(), QErrorNode> {
        cannot_assign_to_const(ctx, left_side)
    }

    fn cannot_assign_to_const(ctx: &mut Context, input: &ExpressionNode) -> Result<(), QErrorNode> {
        if let Locatable {
            element: Expression::Variable(var_name, _),
            ..
        } = input
        {
            if ctx.names.contains_const_recursively(var_name.bare_name()) {
                Err(QError::DuplicateDefinition).with_err_at(input)
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

    pub fn validate(
        left_side: &ExpressionNode,
        right_side: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        if right_side.as_ref().can_cast_to(left_side) {
            Ok(())
        } else {
            Err(QError::TypeMismatch).with_err_at(right_side)
        }
    }
}
