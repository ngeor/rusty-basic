use crate::linter::pre_linter::ResolvedParamTypes;
use crate::linter::{HasFunctions, ResolvedParamType};
use crate::parser::*;
use rusty_common::*;

use super::post_conversion_linter::PostConversionLinter;

pub struct UserDefinedFunctionLinter<'a, R> {
    pub context: &'a R,
}

pub fn lint_call_args(
    args: &ExpressionNodes,
    param_types: &ResolvedParamTypes,
) -> Result<(), QErrorNode> {
    if args.len() != param_types.len() {
        return Err(QError::ArgumentCountMismatch).with_err_no_pos();
    }

    args.iter()
        .zip(param_types.iter())
        .try_for_each(|(a, p)| lint_call_arg(a, p))
}

fn lint_call_arg(
    arg_node: &ExpressionNode,
    param_type: &ResolvedParamType,
) -> Result<(), QErrorNode> {
    let arg = arg_node.as_ref();
    match arg {
        Expression::Variable(_, _)
        | Expression::ArrayElement(_, _, _)
        | Expression::Property(_, _, _) => lint_by_ref_arg(arg_node, param_type),
        _ => lint_by_val_arg(arg_node, param_type),
    }
}

fn lint_by_ref_arg(
    arg_node: &ExpressionNode,
    param_type: &ResolvedParamType,
) -> Result<(), QErrorNode> {
    match param_type {
        ResolvedParamType::BuiltIn(q, _) => {
            lint_arg_node(arg_node, |e| expr_type_matches_type_qualifier_by_ref(e, *q))
        }
        ResolvedParamType::UserDefined(user_defined_type_name) => lint_arg_node(arg_node, |e| {
            expr_type_is_user_defined(e, user_defined_type_name)
        }),
        ResolvedParamType::Array(boxed_element_type) => {
            // we can only pass an array by using the array name followed by parenthesis e.g. `Menu choice$()`
            match arg_node.as_ref() {
                Expression::ArrayElement(
                    name,
                    args,
                    VariableInfo {
                        expression_type, ..
                    },
                ) => {
                    if args.is_empty() {
                        let dummy_expr = Expression::Variable(
                            name.clone(),
                            VariableInfo::new_local(expression_type.clone()),
                        )
                        .at(arg_node);
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

fn lint_by_val_arg(
    arg_node: &ExpressionNode,
    param_type: &ResolvedParamType,
) -> Result<(), QErrorNode> {
    // it's by val, casting is allowed
    if arg_node.as_ref().expression_type().can_cast_to(param_type) {
        Ok(())
    } else {
        Err(QError::ArgumentTypeMismatch).with_err_at(arg_node)
    }
}

fn lint_arg_node<F>(
    arg_node: &ExpressionNode,
    expression_type_predicate: F,
) -> Result<(), QErrorNode>
where
    F: Fn(&ExpressionType) -> bool,
{
    match arg_node_to_expr_type_and_opt_args(arg_node) {
        Some((expression_type, opt_args)) => {
            if has_at_least_one_arg(opt_args) && expression_type_predicate(expression_type) {
                Ok(())
            } else {
                Err(QError::ArgumentTypeMismatch).with_err_at(arg_node)
            }
        }
        _ => Err(QError::ArgumentTypeMismatch).with_err_at(arg_node),
    }
}

fn arg_node_to_expr_type_and_opt_args(
    arg_node: &ExpressionNode,
) -> Option<(&ExpressionType, Option<&ExpressionNodes>)> {
    match arg_node.as_ref() {
        Expression::Variable(
            _,
            VariableInfo {
                expression_type, ..
            },
        )
        | Expression::Property(_, _, expression_type) => Some((expression_type, None)),
        Expression::ArrayElement(
            _,
            args,
            VariableInfo {
                expression_type, ..
            },
        ) => Some((expression_type, Some(args))),
        _ => None,
    }
}

fn has_at_least_one_arg(opt_args: Option<&ExpressionNodes>) -> bool {
    match opt_args {
        Some(args) => !args.is_empty(),
        _ => true, // not an array expression
    }
}

impl<'a, R> UserDefinedFunctionLinter<'a, R>
where
    R: HasFunctions,
{
    fn visit_function(&self, name: &Name, args: &ExpressionNodes) -> Result<(), QErrorNode> {
        if let Name::Qualified(bare_name, qualifier) = name {
            match self.context.functions().get(bare_name) {
                Some(function_signature_node) => {
                    if function_signature_node.as_ref().qualifier() != *qualifier {
                        Err(QError::TypeMismatch).with_err_at(function_signature_node)
                    } else {
                        lint_call_args(args, function_signature_node.as_ref().param_types())
                    }
                }
                None => self.handle_undefined_function(args),
            }
        } else {
            panic!("Unresolved function {:?}", name)
        }
    }

    fn handle_undefined_function(&self, args: &ExpressionNodes) -> Result<(), QErrorNode> {
        for i in 0..args.len() {
            let arg_node = args.get(i).unwrap();
            match arg_node.as_ref().expression_type() {
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

impl<'a, R> PostConversionLinter for UserDefinedFunctionLinter<'a, R>
where
    R: HasFunctions,
{
    fn visit_expression(&mut self, expr_node: &ExpressionNode) -> Result<(), QErrorNode> {
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
