use crate::const_value_resolver::ConstValueResolver;
use crate::converter::context::Context;
use crate::error::{LintError, LintErrorPos};
use crate::QBNumberCast;
use rusty_common::AtPos;
use rusty_parser::ExpressionPos;
use rusty_variant::MAX_INTEGER;

pub fn resolve_string_length(
    ctx: &Context,
    length_expression: &ExpressionPos,
) -> Result<u16, LintErrorPos> {
    let v = ctx.names.resolve_const(length_expression)?;
    let i: i32 = v.try_cast().map_err(|e| e.at(length_expression))?;
    if (1..MAX_INTEGER).contains(&i) {
        Ok(i as u16)
    } else {
        Err(LintError::OutOfStringSpace.at(length_expression))
    }
}
