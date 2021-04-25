use crate::common::*;
use crate::linter::converter::dim_rules::{convert_array_dimensions, resolve_string_length};
use crate::linter::converter::{Context, R};
use crate::linter::type_resolver::TypeResolver;
use crate::parser::*;

pub fn convert(
    ctx: &mut Context,
    bare_name: BareName,
    dim_type: DimType,
    shared: bool,
    pos: Location,
) -> R<DimNameNode> {
    if let DimType::Array(array_dimensions, element_type) = dim_type {
        let dimension_count = array_dimensions.len();
        let (converted_array_dimensions, implicits) =
            convert_array_dimensions(ctx, array_dimensions)?;
        debug_assert_eq!(dimension_count, converted_array_dimensions.len());
        let converted_element_type = to_dim_type(
            ctx,
            &bare_name,
            &converted_array_dimensions,
            element_type.as_ref(),
            pos,
        )?;
        let array_dim_type =
            DimType::Array(converted_array_dimensions, Box::new(converted_element_type));
        ctx.names.insert(
            bare_name.clone(),
            &array_dim_type,
            shared,
            Some(RedimInfo { dimension_count }),
        );
        Ok((DimName::new(bare_name, array_dim_type).at(pos), implicits))
    } else {
        panic!("REDIM without array")
    }
}

fn to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    element_dim_type: &DimType,
    pos: Location,
) -> Result<DimType, QErrorNode> {
    match element_dim_type {
        DimType::Bare => bare_to_dim_type(ctx, bare_name, array_dimensions, pos),
        DimType::BuiltIn(q, built_in_style) => {
            built_in_to_dim_type(ctx, bare_name, array_dimensions, *q, *built_in_style, pos)
        }
        DimType::FixedLengthString(length_expression, resolved_length) => {
            debug_assert_eq!(
                *resolved_length, 0,
                "REDIM string length should not be known"
            );
            fixed_length_string_to_dim_type(
                ctx,
                bare_name,
                array_dimensions,
                length_expression,
                pos,
            )
        }
        DimType::UserDefined(u) => {
            user_defined_type_to_dim_type(ctx, bare_name, array_dimensions, u, pos)
        }
        DimType::Array(_, _) => {
            panic!("REDIM nested array is not supported")
        }
    }
}

fn bare_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    pos: Location,
) -> Result<DimType, QErrorNode> {
    let mut found: Option<(BuiltInStyle, &VariableInfo)> = None;
    let q = ctx.resolve(bare_name);
    for (built_in_style, variable_info) in ctx.names.names_iterator(bare_name) {
        match &variable_info.redim_info {
            Some(r) => {
                if r.dimension_count != array_dimensions.len() {
                    return Err(QError::WrongNumberOfDimensions).with_err_at(pos);
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
                return Err(QError::DuplicateDefinition).with_err_at(pos);
            }
        }
    }
    match found {
        Some((built_in_style, variable_info)) => {
            if let ExpressionType::Array(element_type) = &variable_info.expression_type {
                match element_type.as_ref() {
                    ExpressionType::BuiltIn(q) => Ok(DimType::BuiltIn(*q, built_in_style)),
                    ExpressionType::FixedLengthString(len) => {
                        Ok(DimType::fixed_length_string(*len, pos))
                    }
                    ExpressionType::UserDefined(u) => Ok(DimType::UserDefined(u.clone().at(pos))),
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
    pos: Location,
) -> Result<DimType, QErrorNode> {
    if built_in_style == BuiltInStyle::Compact {
        ctx.names
            .visit_names(bare_name, |built_in_style, variable_info| {
                if built_in_style == BuiltInStyle::Extended {
                    return Err(QError::DuplicateDefinition);
                }
                let opt_q = variable_info.expression_type.opt_qualifier();
                if opt_q.expect("Should be qualified") == q {
                    // other compact arrays of the same name are allowed to co-exist, hence no else block here
                    require_dimension_count(variable_info, array_dimensions.len())?;
                }
                Ok(())
            })
            .with_err_at(pos)?;
    } else {
        ctx.names
            .visit_names(bare_name, |built_in_style, variable_info| {
                if built_in_style == BuiltInStyle::Compact {
                    return Err(QError::DuplicateDefinition);
                }
                require_built_in_array(variable_info, q)?;
                require_dimension_count(variable_info, array_dimensions.len())
            })
            .with_err_at(pos)?;
    }
    Ok(DimType::BuiltIn(q, built_in_style))
}

fn require_built_in_array(variable_info: &VariableInfo, q: TypeQualifier) -> Result<(), QError> {
    if let ExpressionType::Array(element_type) = &variable_info.expression_type {
        if let ExpressionType::BuiltIn(existing_q) = element_type.as_ref() {
            if q == *existing_q {
                return Ok(());
            }
        }
    }
    Err(QError::DuplicateDefinition)
}

fn fixed_length_string_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    length_expression: &ExpressionNode,
    pos: Location,
) -> Result<DimType, QErrorNode> {
    let string_length: u16 = resolve_string_length(ctx, length_expression)?;
    ctx.names
        .visit_names(bare_name, |built_in_style, variable_info| {
            if built_in_style == BuiltInStyle::Compact {
                Err(QError::DuplicateDefinition)
            } else {
                require_fixed_length_string_array(variable_info, string_length)?;
                require_dimension_count(variable_info, array_dimensions.len())
            }
        })
        .with_err_at(pos)?;
    Ok(DimType::fixed_length_string(string_length, pos))
}

fn require_fixed_length_string_array(variable_info: &VariableInfo, len: u16) -> Result<(), QError> {
    if let ExpressionType::Array(element_type) = &variable_info.expression_type {
        if let ExpressionType::FixedLengthString(existing_len) = element_type.as_ref() {
            if len == *existing_len {
                return Ok(());
            }
        }
    }
    Err(QError::DuplicateDefinition)
}

fn user_defined_type_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    array_dimensions: &ArrayDimensions,
    user_defined_type: &BareNameNode,
    pos: Location,
) -> Result<DimType, QErrorNode> {
    ctx.names
        .visit_names(bare_name, |built_in_style, variable_info| {
            if built_in_style == BuiltInStyle::Compact {
                Err(QError::DuplicateDefinition)
            } else {
                require_dimension_count(variable_info, array_dimensions.len()).and_then(|_| {
                    require_user_defined_array(variable_info, user_defined_type.as_ref())
                })
            }
        })
        .with_err_at(pos)?;
    Ok(DimType::UserDefined(user_defined_type.clone()))
}

fn require_dimension_count(
    variable_info: &VariableInfo,
    dimension_count: usize,
) -> Result<(), QError> {
    if let ExpressionType::Array(_) = &variable_info.expression_type {
        match &variable_info.redim_info {
            Some(redim_info) => {
                if redim_info.dimension_count == dimension_count {
                    Ok(())
                } else {
                    Err(QError::WrongNumberOfDimensions)
                }
            }
            _ => Err(QError::ArrayAlreadyDimensioned),
        }
    } else {
        Err(QError::DuplicateDefinition)
    }
}

fn require_user_defined_array(
    variable_info: &VariableInfo,
    user_defined_type: &BareName,
) -> Result<(), QError> {
    if let ExpressionType::Array(element_type) = &variable_info.expression_type {
        if let ExpressionType::UserDefined(u) = element_type.as_ref() {
            if u == user_defined_type {
                return Ok(());
            }
        }
    }
    Err(QError::DuplicateDefinition)
}
