use rusty_common::{AtPos, Position, Positioned};
use rusty_parser::{AsBareName, DimVar, Parameter, TypedName, VarType};

use crate::converter::common::Context;
use crate::core::{ConstLookup, IntoTypeQualifier, LintError, LintErrorPos};

pub fn validate<T>(
    var_name: &TypedName<T>,
    ctx: &Context,
    pos: Position,
) -> Result<(), LintErrorPos>
where
    T: VarType,
    TypedName<T>: CannotClashWithFunctions,
{
    cannot_clash_with_subs(var_name, ctx, pos)?;
    var_name.cannot_clash_with_functions(ctx, pos)?;
    user_defined_type_must_exist(var_name, ctx)?;
    cannot_clash_with_local_constants(var_name, ctx, pos)
}

fn cannot_clash_with_subs<T: VarType>(
    var_name: &TypedName<T>,
    ctx: &Context,
    pos: Position,
) -> Result<(), LintErrorPos> {
    if ctx.subs.contains_key(var_name.as_bare_name()) {
        Err(LintError::DuplicateDefinition.at_pos(pos))
    } else {
        Ok(())
    }
}

fn cannot_clash_with_local_constants<T: VarType>(
    var_name: &TypedName<T>,
    ctx: &Context,
    pos: Position,
) -> Result<(), LintErrorPos> {
    match ctx.names.names().get_const_value(var_name.as_bare_name()) {
        Some(_) => Err(LintError::DuplicateDefinition.at_pos(pos)),
        _ => Ok(()),
    }
}

pub trait CannotClashWithFunctions {
    fn cannot_clash_with_functions(&self, ctx: &Context, pos: Position)
    -> Result<(), LintErrorPos>;
}

impl CannotClashWithFunctions for DimVar {
    fn cannot_clash_with_functions(
        &self,
        ctx: &Context,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        if ctx.functions.contains_key(self.as_bare_name()) {
            Err(LintError::DuplicateDefinition.at_pos(pos))
        } else {
            Ok(())
        }
    }
}

impl CannotClashWithFunctions for Parameter {
    fn cannot_clash_with_functions(
        &self,
        ctx: &Context,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        if let Some(func_qualifier) = ctx.function_qualifier(self.as_bare_name()) {
            if self.var_type().is_extended() {
                Err(LintError::DuplicateDefinition.at_pos(pos))
            } else {
                // for some reason you can have a FUNCTION Add(Add)
                let q = self
                    .var_type()
                    .to_qualifier_recursively()
                    .unwrap_or_else(|| self.as_bare_name().qualify(ctx));
                if q == func_qualifier {
                    Ok(())
                } else {
                    Err(LintError::DuplicateDefinition.at_pos(pos))
                }
            }
        } else {
            Ok(())
        }
    }
}

fn user_defined_type_must_exist<T>(
    var_name: &TypedName<T>,
    ctx: &Context,
) -> Result<(), LintErrorPos>
where
    T: VarType,
{
    match var_name.var_type().as_user_defined_recursively() {
        Some(Positioned {
            element: type_name,
            pos,
        }) => {
            if ctx.user_defined_types.contains_key(type_name) {
                Ok(())
            } else {
                Err(LintError::TypeNotDefined.at(pos))
            }
        }
        _ => Ok(()),
    }
}
