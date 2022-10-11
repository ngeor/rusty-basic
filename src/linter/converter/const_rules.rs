use crate::common::*;
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::converter::Context;
use crate::linter::pre_linter::{HasFunctionView, HasSubView};
use crate::parser::*;
use std::convert::TryFrom;

pub fn on_const(
    ctx: &mut Context,
    left_side: NameNode,
    right_side: ExpressionNode,
) -> Result<(), QErrorNode> {
    const_cannot_clash_with_existing_names(ctx, &left_side)?;
    new_const(ctx, left_side, right_side)
}

fn const_cannot_clash_with_existing_names(
    ctx: &mut Context,
    left_side: &NameNode,
) -> Result<(), QErrorNode> {
    let Locatable {
        element: const_name,
        pos: const_name_pos,
    } = left_side;
    if ctx
        .names
        .contains_any_locally_or_contains_extended_recursively(const_name.bare_name())
        || ctx.subs().contains_key(const_name.bare_name())
        || ctx.functions().contains_key(const_name.bare_name())
    {
        Err(QError::DuplicateDefinition).with_err_at(*const_name_pos)
    } else {
        Ok(())
    }
}

fn new_const(
    ctx: &mut Context,
    left_side: NameNode,
    right_side: ExpressionNode,
) -> Result<(), QErrorNode> {
    let Locatable {
        element: const_name,
        ..
    } = left_side;
    let value_before_casting = ctx.names.resolve_const(&right_side)?;
    let value_qualifier =
        TypeQualifier::try_from(&value_before_casting).with_err_at(&right_side)?;
    let final_value = if const_name.is_bare_or_of_type(value_qualifier) {
        value_before_casting
    } else {
        value_before_casting
            .cast(const_name.qualifier().unwrap())
            .with_err_at(&right_side)?
    };
    ctx.names
        .insert_const(const_name.bare_name().clone(), final_value.clone());
    Ok(())
}
