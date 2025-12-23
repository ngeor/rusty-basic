use crate::const_value_resolver::ConstValueResolver;
use crate::converter::context::Context;
use crate::error::{LintError, LintErrorPos};
use crate::{qualifier_of_const_variant, CastVariant, HasFunctions, HasSubs, LintResult};
use rusty_common::*;
use rusty_parser::specific::*;

pub fn on_const(
    ctx: &mut Context,
    left_side: NamePos,
    right_side: ExpressionPos,
) -> Result<(), LintErrorPos> {
    const_cannot_clash_with_existing_names(ctx, &left_side)?;
    new_const(ctx, left_side, right_side)
}

fn const_cannot_clash_with_existing_names(
    ctx: &mut Context,
    left_side: &NamePos,
) -> Result<(), LintErrorPos> {
    let Positioned {
        element: const_name,
        pos: const_name_pos,
    } = left_side;
    if ctx
        .names
        .contains_any_locally_or_contains_extended_recursively(const_name.bare_name())
        || ctx.subs().contains_key(const_name.bare_name())
        || ctx.functions().contains_key(const_name.bare_name())
    {
        Err(LintError::DuplicateDefinition.at(const_name_pos))
    } else {
        Ok(())
    }
}

fn new_const(
    ctx: &mut Context,
    left_side: NamePos,
    right_side: ExpressionPos,
) -> Result<(), LintErrorPos> {
    let Positioned {
        element: const_name,
        ..
    } = left_side;
    let value_before_casting = ctx.names.resolve_const(&right_side)?;
    let value_qualifier = qualifier_of_const_variant(&value_before_casting);
    let final_value = if const_name.is_bare_or_of_type(value_qualifier) {
        value_before_casting
    } else {
        value_before_casting
            .cast(const_name.qualifier().unwrap())
            .with_err_at(&right_side)?
    };
    ctx.names.insert_const(const_name.into(), final_value);
    Ok(())
}
