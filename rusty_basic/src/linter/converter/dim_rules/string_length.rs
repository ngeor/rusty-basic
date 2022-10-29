use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::context::Context;
use crate::parser::ExpressionNode;
use crate::variant::{QBNumberCast, MAX_INTEGER};
use rusty_common::{QError, QErrorNode, ToLocatableError};

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
