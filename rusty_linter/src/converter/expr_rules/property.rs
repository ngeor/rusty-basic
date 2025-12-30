use rusty_common::{AtPos, Position, Positioned};
use rusty_parser::{
    AsBareName, BareName, ElementType, Expression, ExpressionType, HasExpressionType, Name, ToBareName, UserDefinedType, VariableInfo
};

use crate::converter::common::{Context, ConvertibleIn, ExprContext, ExprContextPos};
use crate::converter::expr_rules::variable::{
    AssignToFunction, ExistingConst, ExistingVar, VarAsUserDefinedFunctionCall, VarResolve, add_as_new_implicit_var
};
use crate::core::{HasUserDefinedTypes, LintError, LintErrorPos};

pub fn convert(
    ctx: &mut Context,
    extra: ExprContextPos,
    left_side: Box<Expression>,
    property_name: Name,
) -> Result<Expression, LintErrorPos> {
    // can we fold it into a name?
    let opt_folded_name = try_fold(&left_side, property_name.clone());
    if let Some(folded_name) = opt_folded_name {
        // checking out if we have an existing variable / const etc that contains a dot
        let mut rules: Vec<Box<dyn VarResolve>> = vec![];
        rules.push(Box::new(ExistingVar::default()));
        if extra.element != ExprContext::Default {
            rules.push(Box::new(AssignToFunction::default()));
        } else {
            // no need to check for built-in, they don't have dots
            rules.push(Box::new(VarAsUserDefinedFunctionCall::default()));
        }
        rules.push(Box::new(ExistingConst::new_local()));
        rules.push(Box::new(ExistingConst::new_recursive()));
        for mut rule in rules {
            if rule.can_handle(ctx, &folded_name) {
                return rule.resolve(ctx, extra, folded_name);
            }
        }
    }

    // it is not a folded name, either it is a property of a known variable or expression,
    // or we need to introduce a new implicit var with a dot
    let unboxed_left_side = *left_side;
    let Positioned {
        element: resolved_left_side,
        ..
    } = unboxed_left_side
        .at_pos(extra.pos)
        .convert_in(ctx, ExprContext::ResolvingPropertyOwner)?;

    // functions cannot return udf so no need to check them
    match &resolved_left_side {
        Expression::Variable(
            _name,
            VariableInfo {
                expression_type, ..
            },
        ) => {
            let temp_expression_type = expression_type.clone();
            existing_property_expression_type(
                ctx,
                extra,
                resolved_left_side,
                &temp_expression_type,
                property_name,
                true,
            )
        }
        Expression::ArrayElement(
            _name,
            _indices,
            VariableInfo {
                expression_type, ..
            },
        ) => {
            let temp_expression_type = expression_type.clone();
            existing_property_expression_type(
                ctx,
                extra,
                resolved_left_side,
                &temp_expression_type,
                property_name,
                false,
            )
        }
        Expression::Property(_left_side, _name, expr_type) => {
            let temp_expression_type = expr_type.clone();
            existing_property_expression_type(
                ctx,
                extra,
                resolved_left_side,
                &temp_expression_type,
                property_name,
                false,
            )
        }
        _ => {
            // this cannot possibly have a dot property
            Err(LintError::TypeMismatch.at_pos(extra.pos))
        }
    }
}

fn try_fold(left_side: &Expression, property_name: Name) -> Option<Name> {
    match left_side.fold_name() {
        Some(left_side_name) => left_side_name.try_concat_name(property_name),
        _ => None,
    }
}

fn existing_property_expression_type(
    ctx: &mut Context,
    extra: ExprContextPos,
    resolved_left_side: Expression,
    expression_type: &ExpressionType,
    property_name: Name,
    allow_unresolved: bool,
) -> Result<Expression, LintErrorPos> {
    match expression_type {
        ExpressionType::UserDefined(user_defined_type_name) => {
            existing_property_user_defined_type_name(
                ctx,
                resolved_left_side,
                user_defined_type_name,
                property_name,
                extra.pos,
            )
        }
        ExpressionType::Unresolved => {
            if allow_unresolved {
                match try_fold(&resolved_left_side, property_name) {
                    Some(folded_name) => Ok(add_as_new_implicit_var(ctx, extra, folded_name)),
                    _ => Err(LintError::TypeMismatch.at_pos(extra.pos)),
                }
            } else {
                Err(LintError::TypeMismatch.at_pos(extra.pos))
            }
        }
        _ => Err(LintError::TypeMismatch.at_pos(extra.pos)),
    }
}

fn existing_property_user_defined_type_name(
    ctx: &Context,
    resolved_left_side: Expression,
    user_defined_type_name: &BareName,
    property_name: Name,
    pos: Position,
) -> Result<Expression, LintErrorPos> {
    match ctx.user_defined_types().get(user_defined_type_name) {
        Some(user_defined_type) => existing_property_user_defined_type(
            resolved_left_side,
            user_defined_type,
            property_name,
            pos,
        ),
        _ => Err(LintError::TypeNotDefined.at_pos(pos)),
    }
}

fn existing_property_user_defined_type(
    resolved_left_side: Expression,
    user_defined_type: &UserDefinedType,
    property_name: Name,
    pos: Position,
) -> Result<Expression, LintErrorPos> {
    match demand_element_by_name(user_defined_type, &property_name) {
        Ok(element_type) => {
            existing_property_element_type(resolved_left_side, element_type, property_name)
        }
        Err(e) => Err(e.at_pos(pos)),
    }
}

fn existing_property_element_type(
    resolved_left_side: Expression,
    element_type: &ElementType,
    property_name: Name,
) -> Result<Expression, LintErrorPos> {
    let bare_name = property_name.to_bare_name();
    let property_name = Name::bare(bare_name);
    Ok(Expression::Property(
        Box::new(resolved_left_side),
        property_name,
        element_type.expression_type(),
    ))
}

fn demand_element_by_name<'a>(
    user_defined_type: &'a UserDefinedType,
    element_name: &Name,
) -> Result<&'a ElementType, LintError> {
    let element_type = find_element_type(user_defined_type, element_name.as_bare_name())
        .ok_or(LintError::ElementNotDefined)?;
    if element_type.can_be_referenced_by_property_name(element_name) {
        Ok(element_type)
    } else {
        Err(LintError::TypeMismatch)
    }
}

fn find_element_type<'a>(
    user_defined_type: &'a UserDefinedType,
    element_name: &BareName,
) -> Option<&'a ElementType> {
    user_defined_type
        .elements()
        .map(|Positioned { element, .. }| element)
        .find(|x| &x.name == element_name)
        .map(|x| &x.element_type)
}
