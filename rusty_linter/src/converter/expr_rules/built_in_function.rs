use rusty_common::Position;
use rusty_parser::{BuiltInFunction, Expression, Expressions};

use crate::converter::expr_rules::function::{
    convert_function_args, functions_must_have_arguments
};
use crate::core::{LintErrorPos, LinterContext};

pub fn convert(
    ctx: &mut LinterContext,
    built_in_function: BuiltInFunction,
    pos: Position,
    args: Expressions,
) -> Result<Expression, LintErrorPos> {
    functions_must_have_arguments(&args, pos)?;
    let converted_args = convert_function_args(ctx, args)?;
    let converted_expr = Expression::BuiltInFunctionCall(built_in_function, converted_args);
    Ok(converted_expr)
}
