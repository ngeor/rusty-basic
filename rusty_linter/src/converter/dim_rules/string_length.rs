use crate::const_value_resolver::ConstValueResolver;
use crate::converter::context::Context;
use crate::QBNumberCast;
use rusty_common::{QError, QErrorNode, ToLocatableError};
use rusty_parser::ExpressionNode;
use rusty_variant::MAX_INTEGER;

pub fn resolve_string_length(
    ctx: &Context,
    length_expression: &ExpressionNode,
) -> Result<u16, QErrorNode> {
    let v = ctx.names.resolve_const(length_expression)?;
    let i: i32 = v.try_cast().with_err_at(length_expression)?;
    if (1..MAX_INTEGER).contains(&i) {
        Ok(i as u16)
    } else {
        Err(QError::OutOfStringSpace).with_err_at(length_expression)
    }
}
