use crate::converter::context::Context;
use crate::error::{LintError, LintErrorPos};
use crate::names::ManyNamesTrait;
use crate::type_resolver::IntoTypeQualifier;
use crate::{HasFunctions, HasSubs, HasUserDefinedTypes, LintResult};
use rusty_common::{AtPos, Positioned};
use rusty_parser::specific::{
    DimVar, Parameter, TypedName, VarTypeIsExtended, VarTypeQualifier,
    VarTypeToUserDefinedRecursively,
};

pub fn validate<T>(var_name: &TypedName<T>, ctx: &Context) -> Result<(), LintErrorPos>
where
    T: VarTypeIsExtended + VarTypeQualifier + VarTypeToUserDefinedRecursively,
    TypedName<T>: CannotClashWithFunctions,
{
    cannot_clash_with_subs(var_name, ctx)?;
    var_name.cannot_clash_with_functions(ctx)?;
    user_defined_type_must_exist(var_name, ctx)?;
    cannot_clash_with_local_constants(var_name, ctx).with_err_no_pos()
}

fn cannot_clash_with_subs<T, C: HasSubs>(
    var_name: &TypedName<T>,
    ctx: &C,
) -> Result<(), LintError> {
    if ctx.subs().contains_key(&var_name.bare_name) {
        Err(LintError::DuplicateDefinition)
    } else {
        Ok(())
    }
}

fn cannot_clash_with_local_constants<T>(
    var_name: &TypedName<T>,
    ctx: &Context,
) -> Result<(), LintError> {
    match ctx.names.names().get_const_value(&var_name.bare_name) {
        Some(_) => Err(LintError::DuplicateDefinition),
        _ => Ok(()),
    }
}

pub trait CannotClashWithFunctions {
    fn cannot_clash_with_functions(&self, ctx: &Context) -> Result<(), LintError>;
}

impl CannotClashWithFunctions for DimVar {
    fn cannot_clash_with_functions(&self, ctx: &Context) -> Result<(), LintError> {
        if ctx.functions().contains_key(&self.bare_name) {
            Err(LintError::DuplicateDefinition)
        } else {
            Ok(())
        }
    }
}

impl CannotClashWithFunctions for Parameter {
    fn cannot_clash_with_functions(&self, ctx: &Context) -> Result<(), LintError> {
        if let Some(func_qualifier) = ctx.function_qualifier(&self.bare_name) {
            if self.var_type.is_extended() {
                Err(LintError::DuplicateDefinition)
            } else {
                // for some reason you can have a FUNCTION Add(Add)
                let q = self
                    .var_type
                    .to_qualifier_recursively()
                    .unwrap_or_else(|| self.bare_name.qualify(ctx));
                if q == func_qualifier {
                    Ok(())
                } else {
                    Err(LintError::DuplicateDefinition)
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
    T: VarTypeToUserDefinedRecursively,
{
    match var_name.var_type.as_user_defined_recursively() {
        Some(Positioned {
            element: type_name,
            pos,
        }) => {
            if ctx.user_defined_types().contains_key(type_name) {
                Ok(())
            } else {
                Err(LintError::TypeNotDefined.at(pos))
            }
        }
        _ => Ok(()),
    }
}
