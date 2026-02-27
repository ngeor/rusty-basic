use rusty_common::*;
use rusty_parser::*;

use crate::converter::common::{Convertible, DimContext, DimNameState};
use crate::core::{
    IntoTypeQualifier, LintError, LintErrorPos, LinterContext, ValidateStringLength
};

pub fn on_dim_type(
    dim_type: DimType,
    bare_name: &BareName,
    ctx: &mut LinterContext,
    extra: DimNameState,
) -> Result<DimType, LintErrorPos> {
    match dim_type {
        DimType::Bare => bare_to_dim_type(ctx, bare_name, extra.pos),
        DimType::BuiltIn(q, built_in_style) => {
            built_in_to_dim_type(ctx, bare_name, q, built_in_style, extra.pos)
        }
        DimType::FixedLengthString(expr, resolved_length) => {
            debug_assert_eq!(resolved_length, 0, "Should not be resolved yet");
            fixed_length_string_to_dim_type(ctx, bare_name, &expr)
        }
        DimType::UserDefined(u) => user_defined_to_dim_type(ctx, bare_name, u, extra.pos),
        DimType::Array(array_dimensions, element_type) => {
            array_to_dim_type(ctx, extra, bare_name, array_dimensions, *element_type)
        }
    }
}

pub fn bare_to_dim_type<T: VarType>(
    ctx: &mut LinterContext,
    bare_name: &BareName,
    pos: Position,
) -> Result<T, LintErrorPos> {
    let resolved_q = bare_name.qualify(ctx);
    require_compact_can_be_defined(ctx, bare_name, resolved_q, pos)?;
    Ok(T::new_built_in_compact(resolved_q))
}

fn require_compact_can_be_defined(
    ctx: &LinterContext,
    bare_name: &BareName,
    q: TypeQualifier,
    pos: Position,
) -> Result<(), LintErrorPos> {
    ctx.names
        .find_name_or_shared_in_parent(bare_name)
        .into_iter()
        .try_for_each(|(built_in_style, variable_info)| {
            if built_in_style == BuiltInStyle::Extended {
                Err(LintError::DuplicateDefinition.at_pos(pos))
            } else {
                let opt_q = variable_info.expression_type.opt_qualifier().unwrap();
                if opt_q == q {
                    Err(LintError::DuplicateDefinition.at_pos(pos))
                } else {
                    Ok(())
                }
            }
        })
}

pub fn built_in_to_dim_type<T: VarType>(
    ctx: &mut LinterContext,
    bare_name: &BareName,
    q: TypeQualifier,
    built_in_style: BuiltInStyle,
    pos: Position,
) -> Result<T, LintErrorPos> {
    match built_in_style {
        BuiltInStyle::Compact => {
            require_compact_can_be_defined(ctx, bare_name, q, pos)?;
            Ok(T::new_built_in_compact(q))
        }
        BuiltInStyle::Extended => {
            require_extended_can_be_defined(ctx, bare_name, pos)?;
            Ok(T::new_built_in_extended(q))
        }
    }
}

fn require_extended_can_be_defined(
    ctx: &LinterContext,
    bare_name: &BareName,
    pos: Position,
) -> Result<(), LintErrorPos> {
    ctx.names
        .find_name_or_shared_in_parent(bare_name)
        .into_iter()
        .try_for_each(|_| Err(LintError::DuplicateDefinition.at_pos(pos)))
}

fn fixed_length_string_to_dim_type(
    ctx: &mut LinterContext,
    bare_name: &BareName,
    length_expression: &ExpressionPos,
) -> Result<DimType, LintErrorPos> {
    require_extended_can_be_defined(ctx, bare_name, length_expression.pos)?;
    let string_length: u16 = length_expression.validate_string_length(&ctx.names)?;
    Ok(DimType::fixed_length_string(
        string_length,
        length_expression.pos(),
    ))
}

pub fn user_defined_to_dim_type<T: VarType>(
    ctx: &mut LinterContext,
    bare_name: &BareName,
    user_defined_type: BareNamePos,
    pos: Position,
) -> Result<T, LintErrorPos> {
    require_extended_can_be_defined(ctx, bare_name, pos)?;
    Ok(T::new_user_defined(user_defined_type))
}

fn array_to_dim_type(
    ctx: &mut LinterContext,
    extra: DimNameState,
    bare_name: &BareName,
    array_dimensions: ArrayDimensions,
    element_type: DimType,
) -> Result<DimType, LintErrorPos> {
    debug_assert!(match extra.dim_context {
        DimContext::Default => {
            !array_dimensions.is_empty()
        }
        _ => true,
    });
    let converted_array_dimensions: ArrayDimensions = array_dimensions.convert(ctx)?;
    let resolved_element_dim_type = on_dim_type(element_type, bare_name, ctx, extra)?;
    Ok(DimType::Array(
        converted_array_dimensions,
        Box::new(resolved_element_dim_type),
    ))
}
