use crate::converter::common::Context;
use crate::core::ConstEvaluator;
use crate::core::{qualifier_of_const_variant, CastVariant, HasSubprograms, LintResult};
use crate::core::{LintError, LintErrorPos};
use rusty_common::*;
use rusty_parser::*;

pub fn on_const(ctx: &mut Context, c: Constant) -> Result<Statement, LintErrorPos> {
    let (left_side, right_side) = c.into();
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
        .contains_any_locally_or_contains_extended_recursively(const_name.as_bare_name())
        || ctx.subs().contains_key(const_name.as_bare_name())
        || ctx.functions().contains_key(const_name.as_bare_name())
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
) -> Result<Statement, LintErrorPos> {
    let Positioned {
        element: const_name,
        pos,
    } = left_side;
    let value_before_casting = ctx.names.eval_const(&right_side)?;
    let value_qualifier = qualifier_of_const_variant(&value_before_casting);
    let final_value = if const_name.is_bare_or_of_type(value_qualifier) {
        value_before_casting
    } else {
        value_before_casting
            .cast(const_name.qualifier().unwrap())
            .with_err_at(&right_side)?
    };
    ctx.names
        .names_mut()
        .insert_const(const_name.as_bare_name().clone(), final_value);

    // here we could return the simplified resolved value of the constant
    // instead of keeping `right_side`.
    // However the `CONST` statement gets ignored anyway. It stored the resolved
    // value in the context, so all expressions that reference it
    // will use it.
    Ok(Statement::constant(const_name.at_pos(pos), right_side))
}
