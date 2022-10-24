use crate::common::*;
use crate::linter::converter::context::*;
use crate::linter::converter::dim_rules::dim_name_state::DimNameState;
use crate::linter::converter::dim_rules::string_length::resolve_string_length;
use crate::linter::converter::traits::Convertible;
use crate::linter::converter::types::DimContext;
use crate::linter::type_resolver::IntoTypeQualifier;
use crate::parser::*;

pub fn on_dim_type<'a, 'b>(
    dim_type: DimType,
    bare_name: &BareName,
    ctx: &mut DimNameState<'a, 'b>,
) -> Result<DimType, QErrorNode> {
    match dim_type {
        DimType::Bare => bare_to_dim_type(ctx, bare_name).with_err_no_pos(),
        DimType::BuiltIn(q, built_in_style) => {
            built_in_to_dim_type(ctx, bare_name, q, built_in_style).with_err_no_pos()
        }
        DimType::FixedLengthString(expr, resolved_length) => {
            debug_assert_eq!(resolved_length, 0, "Should not be resolved yet");
            fixed_length_string_to_dim_type(ctx, bare_name, &expr)
        }
        DimType::UserDefined(u) => user_defined_to_dim_type(ctx, bare_name, u).with_err_no_pos(),
        DimType::Array(array_dimensions, element_type) => {
            array_to_dim_type(ctx, bare_name, array_dimensions, *element_type)
        }
    }
}

pub fn bare_to_dim_type<T: VarTypeNewBuiltInCompact>(
    ctx: &mut Context,
    bare_name: &BareName,
) -> Result<T, QError> {
    let resolved_q = bare_name.qualify(ctx);
    require_compact_can_be_defined(ctx, bare_name, resolved_q)?;
    Ok(T::new_built_in_compact(resolved_q))
}

fn require_compact_can_be_defined(
    ctx: &Context,
    bare_name: &BareName,
    q: TypeQualifier,
) -> Result<(), QError> {
    ctx.names
        .visit_names(bare_name, |built_in_style, variable_info| {
            if built_in_style == BuiltInStyle::Extended {
                Err(QError::DuplicateDefinition)
            } else {
                let opt_q = variable_info.expression_type.opt_qualifier().unwrap();
                if opt_q == q {
                    Err(QError::DuplicateDefinition)
                } else {
                    Ok(())
                }
            }
        })
}

pub fn built_in_to_dim_type<T: VarTypeNewBuiltInCompact + VarTypeNewBuiltInExtended>(
    ctx: &mut Context,
    bare_name: &BareName,
    q: TypeQualifier,
    built_in_style: BuiltInStyle,
) -> Result<T, QError> {
    match built_in_style {
        BuiltInStyle::Compact => {
            require_compact_can_be_defined(ctx, bare_name, q)?;
            Ok(T::new_built_in_compact(q))
        }
        BuiltInStyle::Extended => {
            require_extended_can_be_defined(ctx, bare_name)?;
            Ok(T::new_built_in_extended(q))
        }
    }
}

fn require_extended_can_be_defined(ctx: &Context, bare_name: &BareName) -> Result<(), QError> {
    ctx.names
        .visit_names(bare_name, |_, _| Err(QError::DuplicateDefinition))
}

fn fixed_length_string_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    length_expression: &ExpressionNode,
) -> Result<DimType, QErrorNode> {
    require_extended_can_be_defined(ctx, bare_name).with_err_no_pos()?;
    let string_length: u16 = resolve_string_length(ctx, length_expression)?;
    Ok(DimType::fixed_length_string(
        string_length,
        length_expression.pos(),
    ))
}

pub fn user_defined_to_dim_type<T: VarTypeNewUserDefined>(
    ctx: &mut Context,
    bare_name: &BareName,
    user_defined_type: BareNameNode,
) -> Result<T, QError> {
    require_extended_can_be_defined(ctx, bare_name)?;
    Ok(T::new_user_defined(user_defined_type))
}

fn array_to_dim_type<'a, 'b>(
    ctx: &mut DimNameState<'a, 'b>,
    bare_name: &BareName,
    array_dimensions: ArrayDimensions,
    element_type: DimType,
) -> Result<DimType, QErrorNode> {
    debug_assert!(match ctx.dim_context() {
        DimContext::Default => {
            !array_dimensions.is_empty()
        }
        _ => true,
    });
    let converted_array_dimensions: ArrayDimensions = array_dimensions.convert(ctx)?;
    let resolved_element_dim_type = on_dim_type(element_type, bare_name, ctx)?;
    Ok(DimType::Array(
        converted_array_dimensions,
        Box::new(resolved_element_dim_type),
    ))
}
