use crate::const_value_resolver::ConstValueResolver;
use crate::converter::context::Context;
use crate::QBNumberCast;
use rusty_common::{QError, QErrorPos, WithErrAt};
use rusty_parser::ExpressionPos;
use rusty_variant::MAX_INTEGER;

pub fn resolve_string_length(
    ctx: &Context,
    length_expression: &ExpressionPos,
) -> Result<u16, QErrorPos> {
    let v = ctx.names.resolve_const(length_expression)?;
    let i: i32 = v.try_cast().with_err_at(length_expression)?;
    if (1..MAX_INTEGER).contains(&i) {
        Ok(i as u16)
    } else {
        Err(QError::OutOfStringSpace).with_err_at(length_expression)
    }
}
