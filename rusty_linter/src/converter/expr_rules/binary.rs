use rusty_parser::{Expression, ExpressionPos, Operator};

use crate::converter::common::{ConvertibleIn, ExprContextPos};
use crate::core::{LintErrorPos, LinterContext, binary_cast};

pub fn convert(
    ctx: &mut LinterContext,
    extra: ExprContextPos,
    binary_operator: Operator,
    left: ExpressionPos,
    right: ExpressionPos,
) -> Result<Expression, LintErrorPos> {
    let converted_left = left.convert_in(ctx, extra.element)?;
    let converted_right = right.convert_in(ctx, extra.element)?;
    let new_expr = binary_cast(converted_left, converted_right, binary_operator)?;
    Ok(new_expr)
}
