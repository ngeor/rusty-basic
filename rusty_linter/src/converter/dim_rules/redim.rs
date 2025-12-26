use crate::converter::common::Context;
use crate::converter::common::Convertible;
use crate::converter::common::DimNameState;
use crate::core::IntoTypeQualifier;
use crate::core::ValidateStringLength;
use crate::core::{LintError, LintErrorPos};
use rusty_common::*;
use rusty_parser::*;

pub fn on_redim_type(
    var_type: DimType,
    bare_name: &BareName,
    ctx: &mut Context,
    extra: DimNameState,
) -> Result<(DimType, Option<RedimInfo>), LintErrorPos> {
    if let DimType::Array(array_dimensions, element_type) = var_type {
        let dimension_count = array_dimensions.len();
        let converted_array_dimensions: ArrayDimensions = array_dimensions.convert(ctx)?;
        debug_assert_eq!(dimension_count, converted_array_dimensions.len());
        let converted_element_type = to_dim_type(
            ctx,
            extra,
            bare_name,
            &converted_array_dimensions,
            *element_type,
        )?;
        let array_dim_type =
            DimType::Array(converted_array_dimensions, Box::new(converted_element_type));
        Ok((array_dim_type, Some(RedimInfo { dimension_count })))
    } else {
        panic!("REDIM without array")
    }
}

fn to_dim_type(
    ctx: &mut Context,
    extra: DimNameState,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    element_dim_type: DimType,
) -> Result<DimType, LintErrorPos> {
    match element_dim_type {
        DimType::Bare => bare_to_dim_type(ctx, extra, bare_name, array_dimensions),
        DimType::BuiltIn(q, built_in_style) => built_in_to_dim_type(
            ctx,
            bare_name,
            array_dimensions,
            q,
            built_in_style,
            extra.pos,
        ),
        DimType::FixedLengthString(length_expression, resolved_length) => {
            debug_assert_eq!(
                resolved_length, 0,
                "REDIM string length should not be known"
            );
            fixed_length_string_to_dim_type(
                ctx,
                extra,
                bare_name,
                array_dimensions,
                &length_expression,
            )
        }
        DimType::UserDefined(u) => {
            user_defined_type_to_dim_type(ctx, bare_name, array_dimensions, u, extra.pos)
        }
        DimType::Array(_, _) => {
            panic!("REDIM nested array is not supported")
        }
    }
}

fn bare_to_dim_type(
    ctx: &mut Context,
    extra: DimNameState,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
) -> Result<DimType, LintErrorPos> {
    let mut found: Option<(BuiltInStyle, &VariableInfo)> = None;
    let q = bare_name.qualify(ctx);
    for (built_in_style, variable_info) in ctx.names.find_name_or_shared_in_parent(bare_name) {
        match &variable_info.redim_info {
            Some(r) => {
                if r.dimension_count != array_dimensions.len() {
                    return Err(LintError::WrongNumberOfDimensions.at_pos(extra.pos));
                }

                match built_in_style {
                    BuiltInStyle::Compact => {
                        let opt_q: Option<TypeQualifier> =
                            variable_info.expression_type.opt_qualifier();
                        let existing_q = opt_q.expect("Should be qualified");
                        if existing_q == q {
                            debug_assert!(found.is_none());
                            found = Some((built_in_style, variable_info));
                        }
                    }
                    BuiltInStyle::Extended => {
                        debug_assert!(found.is_none());
                        found = Some((built_in_style, variable_info));
                    }
                }
            }
            _ => {
                return Err(LintError::DuplicateDefinition.at_pos(extra.pos));
            }
        }
    }
    match found {
        Some((built_in_style, variable_info)) => {
            if let ExpressionType::Array(element_type) = &variable_info.expression_type {
                match element_type.as_ref() {
                    ExpressionType::BuiltIn(q) => Ok(DimType::BuiltIn(*q, built_in_style)),
                    ExpressionType::FixedLengthString(len) => {
                        Ok(DimType::fixed_length_string(*len, extra.pos))
                    }
                    ExpressionType::UserDefined(u) => {
                        Ok(DimType::UserDefined(u.clone().at_pos(extra.pos)))
                    }
                    _ => {
                        panic!("REDIM with nested array or unresolved type");
                    }
                }
            } else {
                panic!("REDIM without array");
            }
        }
        None => Ok(DimType::BuiltIn(q, BuiltInStyle::Compact)),
    }
}

fn built_in_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    q: TypeQualifier,
    built_in_style: BuiltInStyle,
    pos: Position,
) -> Result<DimType, LintErrorPos> {
    let mut it = ctx
        .names
        .find_name_or_shared_in_parent(bare_name)
        .into_iter();
    if built_in_style == BuiltInStyle::Compact {
        it.try_for_each(|(built_in_style, variable_info)| {
            if built_in_style == BuiltInStyle::Extended {
                return Err(LintError::DuplicateDefinition.at_pos(pos));
            }
            let opt_q = variable_info.expression_type.opt_qualifier();
            if opt_q.expect("Should be qualified") == q {
                // other compact arrays of the same name are allowed to co-exist, hence no else block here
                require_dimension_count(variable_info, array_dimensions.len(), pos)?;
            }
            Ok(())
        })?;
    } else {
        it.try_for_each(|(built_in_style, variable_info)| {
            if built_in_style == BuiltInStyle::Compact {
                return Err(LintError::DuplicateDefinition.at_pos(pos));
            }
            require_built_in_array(variable_info, q, pos)?;
            require_dimension_count(variable_info, array_dimensions.len(), pos)
        })?;
    }
    Ok(DimType::BuiltIn(q, built_in_style))
}

fn require_built_in_array(
    variable_info: &VariableInfo,
    q: TypeQualifier,
    pos: Position,
) -> Result<(), LintErrorPos> {
    if let ExpressionType::Array(element_type) = &variable_info.expression_type {
        if let ExpressionType::BuiltIn(existing_q) = element_type.as_ref() {
            if q == *existing_q {
                return Ok(());
            }
        }
    }
    Err(LintError::DuplicateDefinition.at_pos(pos))
}

fn fixed_length_string_to_dim_type(
    ctx: &mut Context,
    extra: DimNameState,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    length_expression: &ExpressionPos,
) -> Result<DimType, LintErrorPos> {
    let string_length: u16 = length_expression.validate_string_length(&ctx.names)?;
    ctx.names
        .find_name_or_shared_in_parent(bare_name)
        .into_iter()
        .try_for_each(|(built_in_style, variable_info)| {
            if built_in_style == BuiltInStyle::Compact {
                Err(LintError::DuplicateDefinition.at_pos(extra.pos))
            } else {
                require_fixed_length_string_array(variable_info, string_length, extra.pos)?;
                require_dimension_count(variable_info, array_dimensions.len(), extra.pos)
            }
        })?;
    Ok(DimType::fixed_length_string(string_length, extra.pos))
}

fn require_fixed_length_string_array(
    variable_info: &VariableInfo,
    len: u16,
    pos: Position,
) -> Result<(), LintErrorPos> {
    if let ExpressionType::Array(element_type) = &variable_info.expression_type {
        if let ExpressionType::FixedLengthString(existing_len) = element_type.as_ref() {
            if len == *existing_len {
                return Ok(());
            }
        }
    }
    Err(LintError::DuplicateDefinition.at_pos(pos))
}

fn user_defined_type_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    user_defined_type: BareNamePos,
    pos: Position,
) -> Result<DimType, LintErrorPos> {
    ctx.names
        .find_name_or_shared_in_parent(bare_name)
        .into_iter()
        .try_for_each(|(built_in_style, variable_info)| {
            if built_in_style == BuiltInStyle::Compact {
                Err(LintError::DuplicateDefinition.at_pos(pos))
            } else {
                require_dimension_count(variable_info, array_dimensions.len(), pos).and_then(|_| {
                    require_user_defined_array(variable_info, &user_defined_type.element, pos)
                })
            }
        })?;
    Ok(DimType::UserDefined(user_defined_type))
}

fn require_dimension_count(
    variable_info: &VariableInfo,
    dimension_count: usize,
    pos: Position,
) -> Result<(), LintErrorPos> {
    if let ExpressionType::Array(_) = &variable_info.expression_type {
        match &variable_info.redim_info {
            Some(redim_info) => {
                if redim_info.dimension_count == dimension_count {
                    Ok(())
                } else {
                    Err(LintError::WrongNumberOfDimensions.at_pos(pos))
                }
            }
            _ => Err(LintError::ArrayAlreadyDimensioned.at_pos(pos)),
        }
    } else {
        Err(LintError::DuplicateDefinition.at_pos(pos))
    }
}

fn require_user_defined_array(
    variable_info: &VariableInfo,
    user_defined_type: &BareName,
    pos: Position,
) -> Result<(), LintErrorPos> {
    if let ExpressionType::Array(element_type) = &variable_info.expression_type {
        if let ExpressionType::UserDefined(u) = element_type.as_ref() {
            if u == user_defined_type {
                return Ok(());
            }
        }
    }
    Err(LintError::DuplicateDefinition.at_pos(pos))
}
