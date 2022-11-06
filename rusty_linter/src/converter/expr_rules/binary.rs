use crate::casting::binary_cast;
use crate::converter::expr_rules::*;
use crate::error::LintErrorPos;

pub fn convert(
    ctx: &mut PosExprState,
    binary_operator: Operator,
    left: ExpressionPos,
    right: ExpressionPos,
) -> Result<Expression, LintErrorPos> {
    let converted_left = left.convert(ctx)?;
    let converted_right = right.convert(ctx)?;
    let new_expr = binary_cast(converted_left, converted_right, binary_operator)?;
    Ok(new_expr)
}
