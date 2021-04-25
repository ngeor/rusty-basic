use crate::common::*;
use crate::linter::converter::dim_rules::{
    convert_array_dimensions, no_implicits, resolve_string_length,
};
use crate::linter::converter::names::Visitor;
use crate::linter::converter::{Context, R};
use crate::linter::type_resolver::TypeResolver;
use crate::linter::DimContext;
use crate::parser::*;

pub fn do_convert_default(
    ctx: &mut Context,
    bare_name: BareName,
    dim_type: DimType,
    dim_context: DimContext,
    shared: bool,
    pos: Location,
) -> R<DimNameNode> {
    debug_assert_ne!(dim_context, DimContext::Redim);
    resolve_dim_type_default(ctx, &bare_name, &dim_type, dim_context, shared, pos).map(
        |(dim_type, implicits)| {
            ctx.names.insert(bare_name.clone(), &dim_type, shared, None);
            (DimName::new(bare_name, dim_type).at(pos), implicits)
        },
    )
}

fn resolve_dim_type_default(
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

struct BuiltInCompactVisitor(TypeQualifier);

impl Visitor for BuiltInCompactVisitor {
    fn on_compact(
        &mut self,
        q: TypeQualifier,
        _variable_info: &VariableInfo,
    ) -> Result<(), QError> {
        if self.0 == q {
            Err(QError::DuplicateDefinition)
        } else {
            Ok(())
        }
    }

    fn on_extended(&mut self, _variable_info: &VariableInfo) -> Result<(), QError> {
        Err(QError::DuplicateDefinition)
    }
}

fn bare_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    pos: Location,
) -> Result<DimType, QErrorNode> {
    let resolved_q = ctx.resolve(bare_name);
    let mut visitor = BuiltInCompactVisitor(resolved_q);
    ctx.names.visit(bare_name, &mut visitor).with_err_at(pos)?;
    Ok(DimType::BuiltIn(resolved_q, BuiltInStyle::Compact))
}

struct ExtendedVisitor;

impl Visitor for ExtendedVisitor {
    fn on_compact(
        &mut self,
        _q: TypeQualifier,
        _variable_info: &VariableInfo,
    ) -> Result<(), QError> {
        Err(QError::DuplicateDefinition)
    }

    fn on_extended(&mut self, _variable_info: &VariableInfo) -> Result<(), QError> {
        Err(QError::DuplicateDefinition)
    }
}

fn built_in_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    q: TypeQualifier,
    built_in_style: BuiltInStyle,
    pos: Location,
) -> Result<DimType, QErrorNode> {
    match built_in_style {
        BuiltInStyle::Compact => {
            let mut visitor = BuiltInCompactVisitor(q);
            ctx.names.visit(bare_name, &mut visitor).with_err_at(pos)?;
            Ok(DimType::BuiltIn(q, BuiltInStyle::Compact))
        }
        BuiltInStyle::Extended => {
            let mut visitor = ExtendedVisitor;
            ctx.names.visit(bare_name, &mut visitor).with_err_at(pos)?;
            Ok(DimType::BuiltIn(q, BuiltInStyle::Extended))
        }
    }
}

fn fixed_length_string_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    length_expression: &ExpressionNode,
    pos: Location,
) -> Result<DimType, QErrorNode> {
    for (_, _) in ctx.names.names_iterator(bare_name) {
        return Err(QError::DuplicateDefinition).with_err_at(pos);
    }
    let string_length: u16 = resolve_string_length(ctx, length_expression)?;
    Ok(DimType::fixed_length_string(
        string_length,
        length_expression.pos(),
    ))
}

fn user_defined_to_dim_type(
    ctx: &mut Context,
    bare_name: &BareName,
    user_defined_type: &BareNameNode,
    pos: Location,
) -> Result<DimType, QErrorNode> {
    let mut visitor = ExtendedVisitor;
    ctx.names.visit(bare_name, &mut visitor).with_err_at(pos)?;
    Ok(DimType::UserDefined(user_defined_type.clone()))
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
        resolve_dim_type_default(ctx, bare_name, element_type, dim_context, shared, pos)?;
    implicits.append(&mut resolved_implicits);
    let array_dim_type = DimType::Array(
        converted_array_dimensions,
        Box::new(resolved_element_dim_type),
    );
    Ok((array_dim_type, implicits))
}
