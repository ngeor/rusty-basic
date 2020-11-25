use super::post_conversion_linter::PostConversionLinter;
use crate::common::*;
use crate::parser::{
    BareName, Expression, ExpressionNode, ExpressionType, FunctionMap, HasExpressionType, Name,
    ParamType, ParamTypes, QualifiedName, TypeQualifier,
};

pub struct UserDefinedFunctionLinter<'a> {
    pub functions: &'a FunctionMap,
}

pub fn lint_call_args(
    args: &Vec<ExpressionNode>,
    param_types: &ParamTypes,
) -> Result<(), QErrorNode> {
    if args.len() != param_types.len() {
        return err_no_pos(QError::ArgumentCountMismatch);
    }

    args.iter()
        .zip(param_types.iter())
        .map(|(a, p)| lint_call_arg(a, p))
        .collect()
}

fn lint_call_arg(arg_node: &ExpressionNode, param_type: &ParamType) -> Result<(), QErrorNode> {
    let arg = arg_node.as_ref();
    match arg {
        Expression::Variable(_, _)
        | Expression::ArrayElement(_, _, _)
        | Expression::Property(_, _, _) => lint_by_ref_arg(arg_node, param_type),
        _ => lint_by_val_arg(arg_node, param_type),
    }
}

fn lint_by_ref_arg(arg_node: &ExpressionNode, param_type: &ParamType) -> Result<(), QErrorNode> {
    match param_type {
        ParamType::Bare => panic!("Unresolved param {:?} {:?}", arg_node, param_type),
        ParamType::BuiltIn(q, _) => match arg_node.as_ref() {
            Expression::Variable(_, expr_type) | Expression::Property(_, _, expr_type) => {
                // either match or dollar string + FixedLengthString
                if expr_type_matches_type_qualifier_by_ref(expr_type, *q) {
                    Ok(())
                } else {
                    Err(QError::ArgumentTypeMismatch).with_err_at(arg_node)
                }
            }
            Expression::ArrayElement(_, args, expr_type) => {
                // must have at least one arg otherwise it is full array
                if !args.is_empty() && expr_type_matches_type_qualifier_by_ref(expr_type, *q) {
                    Ok(())
                } else {
                    Err(QError::ArgumentTypeMismatch).with_err_at(arg_node)
                }
            }
            _ => Err(QError::ArgumentTypeMismatch).with_err_at(arg_node),
        },
        ParamType::UserDefined(Locatable {
            element: user_defined_type_name,
            ..
        }) => match arg_node.as_ref() {
            Expression::Variable(_, expr_type) | Expression::Property(_, _, expr_type) => {
                if expr_type_is_user_defined(expr_type, user_defined_type_name) {
                    Ok(())
                } else {
                    Err(QError::ArgumentTypeMismatch).with_err_at(arg_node)
                }
            }
            Expression::ArrayElement(_, args, expr_type) => {
                if !args.is_empty() && expr_type_is_user_defined(expr_type, user_defined_type_name)
                {
                    Ok(())
                } else {
                    Err(QError::ArgumentTypeMismatch).with_err_at(arg_node)
                }
            }
            _ => Err(QError::ArgumentTypeMismatch).with_err_at(arg_node),
        },
        ParamType::Array(boxed_element_type) => {
            // we can only pass an array by using the array name followed by parenthesis e.g. `Menu choice$()`
            match arg_node.as_ref() {
                Expression::ArrayElement(name, args, expr_type) => {
                    if args.is_empty() {
                        let dummy_expr =
                            Expression::Variable(name.clone(), expr_type.clone()).at(arg_node);
                        lint_by_ref_arg(&dummy_expr, boxed_element_type.as_ref())
                    } else {
                        Err(QError::ArgumentTypeMismatch).with_err_at(arg_node)
                    }
                }
                _ => Err(QError::ArgumentTypeMismatch).with_err_at(arg_node),
            }
        }
    }
}

fn expr_type_matches_type_qualifier_by_ref(expr_type: &ExpressionType, q: TypeQualifier) -> bool {
    match expr_type {
        ExpressionType::BuiltIn(expr_q) => *expr_q == q,
        ExpressionType::FixedLengthString(_) => q == TypeQualifier::DollarString,
        _ => false,
    }
}

fn expr_type_is_user_defined(
    expr_type: &ExpressionType,
    user_defined_type_name: &BareName,
) -> bool {
    match expr_type {
        ExpressionType::UserDefined(expr_u) => expr_u == user_defined_type_name,
        _ => false,
    }
}

fn lint_by_val_arg(arg_node: &ExpressionNode, param_type: &ParamType) -> Result<(), QErrorNode> {
    // it's by val, casting is allowed
    if arg_node.as_ref().can_cast_to(param_type) {
        Ok(())
    } else {
        Err(QError::ArgumentTypeMismatch).with_err_at(arg_node)
    }
}

impl<'a> UserDefinedFunctionLinter<'a> {
    fn visit_function(&self, name: &Name, args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if let Name::Qualified(qualified_name) = name {
            let QualifiedName {
                bare_name,
                qualifier,
            } = qualified_name;
            match self.functions.get(bare_name) {
                Some(Locatable {
                    element: (return_type, param_types),
                    ..
                }) => {
                    if return_type != qualifier {
                        err_no_pos(QError::TypeMismatch)
                    } else {
                        lint_call_args(args, param_types)
                    }
                }
                None => self.handle_undefined_function(args),
            }
        } else {
            panic!("Unresolved function {:?}", name)
        }
    }

    fn handle_undefined_function(&self, args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        for i in 0..args.len() {
            let arg_node = args.get(i).unwrap();
            match arg_node.expression_type() {
                ExpressionType::BuiltIn(q) => {
                    if q == TypeQualifier::DollarString {
                        return Err(QError::ArgumentTypeMismatch).with_err_at(arg_node);
                    }
                }
                _ => {
                    return Err(QError::ArgumentTypeMismatch).with_err_at(arg_node);
                }
            }
        }

        // is converted to a literal 0 in undefined_function_reducer
        Ok(())
    }
}

impl<'a> PostConversionLinter for UserDefinedFunctionLinter<'a> {
    fn visit_expression(&self, expr_node: &ExpressionNode) -> Result<(), QErrorNode> {
        let Locatable { element: e, pos } = expr_node;
        match e {
            Expression::FunctionCall(n, args) => {
                for x in args {
                    self.visit_expression(x)?;
                }
                self.visit_function(n, args).patch_err_pos(pos)
            }
            Expression::BinaryExpression(_, left, right, _) => {
                self.visit_expression(left)?;
                self.visit_expression(right)
            }
            Expression::UnaryExpression(_, child) => self.visit_expression(child),
            _ => Ok(()),
        }
    }
}
