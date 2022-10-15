use crate::common::*;
use crate::linter::converter::dim_rules::{
    convert_array_dimensions, no_implicits, resolve_string_length,
};
use crate::linter::converter::{Context, R};
use crate::linter::type_resolver::IntoTypeQualifier;
use crate::linter::DimContext;
use crate::parser::*;

pub fn convert(
    ctx: &mut Context,
    bare_name: BareName,
    dim_type: DimType,
    dim_context: DimContext,
    shared: bool,
    pos: Location,
) -> R<DimNameNode> {
    debug_assert_ne!(dim_context, DimContext::Redim);
    to_dim_type(ctx, &bare_name, &dim_type, dim_context, shared, pos).map(
        |(dim_type, implicits)| {
            ctx.names.insert(bare_name.clone(), &dim_type, shared, None);
            (DimName::new(bare_name, dim_type).at(pos), implicits)
        },
    )
}

fn to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    dim_type: &DimType,
    dim_context: DimContext,
    shared: bool,
    pos: Location,
) -> R<DimType> {
    debug_assert_ne!(dim_context, DimContext::Redim);
    match dim_type {
        DimType::Bare => bare_to_dim_type(ctx, bare_name, pos).map(no_implicits),
        DimType::BuiltIn(q, built_in_style) => {
            built_in_to_dim_type(ctx, bare_name, *q, *built_in_style, pos).map(no_implicits)
        }
        DimType::FixedLengthString(length_expression, resolved_length) => {
            debug_assert_eq!(*resolved_length, 0, "Should not be resolved yet");
            fixed_length_string_to_dim_type(ctx, bare_name, length_expression, pos)
                .map(no_implicits)
        }
        DimType::UserDefined(u) => {
            user_defined_to_dim_type(ctx, bare_name, u, pos).map(no_implicits)
        }
        DimType::Array(array_dimensions, element_type) => array_to_dim_type(
            ctx,
            bare_name,
            array_dimensions,
            element_type.as_ref(),
            dim_context,
            shared,
            pos,
        ),
    }
}

fn bare_to_dim_type<T: VarTypeNewBuiltInCompact>(
    ctx: &mut Context,
    bare_name: &BareName,
    pos: Location,
) -> Result<T, QErrorNode> {
    let resolved_q = bare_name.qualify(ctx);
    require_compact_can_be_defined(ctx, bare_name, resolved_q).with_err_at(pos)?;
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

fn built_in_to_dim_type<T: VarTypeNewBuiltInCompact + VarTypeNewBuiltInExtended>(
    ctx: &mut Context,
    bare_name: &BareName,
    q: TypeQualifier,
    built_in_style: BuiltInStyle,
    pos: Location,
) -> Result<T, QErrorNode> {
    match built_in_style {
        BuiltInStyle::Compact => {
            require_compact_can_be_defined(ctx, bare_name, q).with_err_at(pos)?;
            Ok(T::new_built_in_compact(q))
        }
        BuiltInStyle::Extended => {
            require_extended_can_be_defined(ctx, bare_name).with_err_at(pos)?;
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
    pos: Location,
) -> Result<DimType, QErrorNode> {
    require_extended_can_be_defined(ctx, bare_name).with_err_at(pos)?;
    let string_length: u16 = resolve_string_length(ctx, length_expression)?;
    Ok(DimType::fixed_length_string(
        string_length,
        length_expression.pos(),
    ))
}

fn user_defined_to_dim_type<T: VarTypeNewUserDefined>(
    ctx: &mut Context,
    bare_name: &BareName,
    user_defined_type: &BareNameNode,
    pos: Location,
) -> Result<T, QErrorNode> {
    require_extended_can_be_defined(ctx, bare_name).with_err_at(pos)?;
    Ok(T::new_user_defined(user_defined_type.clone()))
}

fn array_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    element_type: &DimType,
    dim_context: DimContext,
    shared: bool,
    pos: Location,
) -> R<DimType> {
    debug_assert!(match dim_context {
        DimContext::Default => {
            array_dimensions.len() > 0
        }
        DimContext::Param => {
            array_dimensions.is_empty()
        }
        _ => true,
    });
    // TODO optimize array_dimensions.clone()
    let (converted_array_dimensions, mut implicits) =
        convert_array_dimensions(ctx, array_dimensions.clone())?;
    let (resolved_element_dim_type, mut resolved_implicits) =
        to_dim_type(ctx, bare_name, element_type, dim_context, shared, pos)?;
    implicits.append(&mut resolved_implicits);
    let array_dim_type = DimType::Array(
        converted_array_dimensions,
        Box::new(resolved_element_dim_type),
    );
    Ok((array_dim_type, implicits))
}
