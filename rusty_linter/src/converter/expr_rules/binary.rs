use rusty_parser::{Expression, ExpressionPos, Operator};

use crate::converter::common::{Context, ConvertibleIn, ExprContextPos};
use crate::core::{LintErrorPos, binary_cast};

pub fn convert(
    ctx: &mut Context,
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
