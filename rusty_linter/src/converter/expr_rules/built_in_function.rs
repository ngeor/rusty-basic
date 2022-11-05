use crate::converter::expr_rules::function::{
    convert_function_args, functions_must_have_arguments,
};
use crate::converter::expr_rules::*;

pub fn convert(
    ctx: &mut Context,
    built_in_function: BuiltInFunction,
    args: Expressions,
) -> Result<Expression, QErrorPos> {
    functions_must_have_arguments(&args)?;
    let converted_args = convert_function_args(ctx, args)?;
    let converted_expr = Expression::BuiltInFunctionCall(built_in_function, converted_args);
    Ok(converted_expr)
}
