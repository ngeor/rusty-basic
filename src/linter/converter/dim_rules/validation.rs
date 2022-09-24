use crate::common::*;
use crate::linter::converter::Context;
use crate::linter::type_resolver::TypeResolver;
use crate::linter::DimContext;
use crate::parser::*;

pub fn validate(
    ctx: &Context,
    bare_name: &BareName,
    dim_type: &DimType,
    dim_context: DimContext,
    shared: bool,
) -> Result<(), QErrorNode> {
    cannot_clash_with_subs(ctx, bare_name).with_err_no_pos()?;
    cannot_clash_with_functions(ctx, bare_name, dim_type, dim_context).with_err_no_pos()?;
    user_defined_type_must_exist(ctx, dim_type)?;
    cannot_clash_with_local_constants(ctx, bare_name).with_err_no_pos()?;
    shared_validation(ctx, dim_context, shared).with_err_no_pos()
}

fn cannot_clash_with_subs(ctx: &Context, bare_name: &BareName) -> Result<(), QError> {
    if ctx.subs.contains_key(bare_name) {
        Err(QError::DuplicateDefinition)
    } else {
        Ok(())
    }
}

fn cannot_clash_with_local_constants(ctx: &Context, bare_name: &BareName) -> Result<(), QError> {
    if ctx.names.contains_const(bare_name) {
        Err(QError::DuplicateDefinition)
    } else {
        Ok(())
    }
}

fn cannot_clash_with_functions(
    ctx: &Context,
    bare_name: &BareName,
    dim_type: &DimType,
    dim_context: DimContext,
) -> Result<(), QError> {
    if dim_context == DimContext::Param {
        if let Some(func_qualifier) = ctx.function_qualifier(bare_name) {
            if dim_type.is_extended() {
                Err(QError::DuplicateDefinition)
            } else {
                // for some reason you can have a FUNCTION Add(Add)
                let q = ctx.resolve_dim_name_to_qualifier(bare_name, dim_type);
                if q == func_qualifier {
                    Ok(())
                } else {
                    Err(QError::DuplicateDefinition)
                }
            }
        } else {
            Ok(())
        }
    } else {
        if ctx.functions.contains_key(bare_name) {
            Err(QError::DuplicateDefinition)
        } else {
            Ok(())
        }
    }
}

fn user_defined_type_must_exist(ctx: &Context, dim_type: &DimType) -> Result<(), QErrorNode> {
    match dim_type {
        DimType::UserDefined(Locatable {
            element: type_name,
            pos,
        }) => {
            if ctx.user_defined_types.contains_key(type_name) {
                Ok(())
            } else {
                Err(QError::TypeNotDefined).with_err_at(*pos)
            }
        }
        DimType::Array(_, element_type) => user_defined_type_must_exist(ctx, element_type.as_ref()),
        _ => Ok(()),
    }
}

fn shared_validation(ctx: &Context, dim_context: DimContext, shared: bool) -> Result<(), QError> {
    if shared {
        // this should not happen based on the parser
        debug_assert_ne!(dim_context, DimContext::Param);
        if ctx.is_in_subprogram() {
            return Err(QError::IllegalInSubFunction);
        }
    }
    Ok(())
}
