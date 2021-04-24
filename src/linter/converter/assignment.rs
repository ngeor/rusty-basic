use crate::common::{AtLocation, Locatable, QErrorNode};
use crate::linter::converter::{Context, ConverterImpl, ExprContext, Implicits, R};
use crate::parser::{ExpressionNode, Statement, StatementNode};

impl<'a> ConverterImpl<'a> {
    pub fn assignment(
        &mut self,
        name_expr_node: ExpressionNode,
        expression_node: ExpressionNode,
    ) -> R<StatementNode> {
        self.context
            .on_assignment(name_expr_node, expression_node)
            .map(|(Locatable { element: left, pos }, right, implicit_vars)| {
                (Statement::Assignment(left, right).at(pos), implicit_vars)
            })
    }
}

pub fn on_assignment(
    context: &mut Context,
    left_side: ExpressionNode,
    right_side: ExpressionNode,
) -> Result<(ExpressionNode, ExpressionNode, Implicits), QErrorNode> {
    assignment_pre_conversion_validation_rules::validate(context, &left_side)?;
    let (converted_right_side, mut right_side_implicit_vars) =
        context.on_expression(right_side, ExprContext::Default)?;
    let (converted_left_side, mut left_side_implicit_vars) =
        context.on_expression(left_side, ExprContext::Assignment)?;
    assignment_post_conversion_validation_rules::validate(
        &converted_left_side,
        &converted_right_side,
    )?;
    left_side_implicit_vars.append(&mut right_side_implicit_vars);
    Ok((
        converted_left_side,
        converted_right_side,
        left_side_implicit_vars,
    ))
}

mod assignment_pre_conversion_validation_rules {
    use crate::common::{QError, ToLocatableError};
    use crate::parser::Expression;

    use super::*;

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
    use crate::common::{CanCastTo, QError, ToLocatableError};

    use super::*;

    pub fn validate(
        left_side: &ExpressionNode,
        right_side: &ExpressionNode,
    ) -> Result<(), QErrorNode> {
        if right_side.can_cast_to(left_side) {
            Ok(())
        } else {
            Err(QError::TypeMismatch).with_err_at(right_side)
        }
    }
}
