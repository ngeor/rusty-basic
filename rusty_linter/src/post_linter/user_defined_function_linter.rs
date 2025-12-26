use crate::core::*;
use rusty_common::*;
use rusty_parser::*;

use super::post_conversion_linter::PostConversionLinter;

pub struct UserDefinedFunctionLinter<'a, R> {
    pub linter_context: &'a R,
}

pub fn lint_call_args(
    args: &Expressions,
    param_types: &ResolvedParamTypes,
    pos: Position,
) -> Result<(), LintErrorPos> {
    if args.len() != param_types.len() {
        return Err(LintError::ArgumentCountMismatch.at_pos(pos));
    }

    args.iter()
        .zip(param_types.iter())
        .try_for_each(|(a, p)| lint_call_arg(a, p))
}

fn lint_call_arg(
    arg_pos: &ExpressionPos,
    param_type: &ResolvedParamType,
) -> Result<(), LintErrorPos> {
    match &arg_pos.element {
        Expression::Variable(_, _)
        | Expression::ArrayElement(_, _, _)
        | Expression::Property(_, _, _) => lint_by_ref_arg(arg_pos, param_type),
        _ => lint_by_val_arg(arg_pos, param_type),
    }
}

fn lint_by_ref_arg(
    arg_pos: &ExpressionPos,
    param_type: &ResolvedParamType,
) -> Result<(), LintErrorPos> {
    match param_type {
        ResolvedParamType::BuiltIn(q, _) => {
            lint_arg_pos(arg_pos, |e| expr_type_matches_type_qualifier_by_ref(e, *q))
        }
        ResolvedParamType::UserDefined(user_defined_type_name) => lint_arg_pos(arg_pos, |e| {
            expr_type_is_user_defined(e, user_defined_type_name)
        }),
        ResolvedParamType::Array(boxed_element_type) => {
            // we can only pass an array by using the array name followed by parenthesis e.g. `Menu choice$()`
            match &arg_pos.element {
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
                        .at(arg_pos);
                        lint_by_ref_arg(&dummy_expr, boxed_element_type.as_ref())
                    } else {
                        Err(LintError::ArgumentTypeMismatch.at(arg_pos))
                    }
                }
                _ => Err(LintError::ArgumentTypeMismatch.at(arg_pos)),
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
    arg_pos: &ExpressionPos,
    param_type: &ResolvedParamType,
) -> Result<(), LintErrorPos> {
    // it's by val, casting is allowed
    if arg_pos.expression_type().can_cast_to(param_type) {
        Ok(())
    } else {
        Err(LintError::ArgumentTypeMismatch.at(arg_pos))
    }
}

fn lint_arg_pos<F>(
    arg_pos: &ExpressionPos,
    expression_type_predicate: F,
) -> Result<(), LintErrorPos>
where
    F: Fn(&ExpressionType) -> bool,
{
    match arg_pos_to_expr_type_and_opt_args(arg_pos) {
        Some((expression_type, opt_args)) => {
            if has_at_least_one_arg(opt_args) && expression_type_predicate(expression_type) {
                Ok(())
            } else {
                Err(LintError::ArgumentTypeMismatch.at(arg_pos))
            }
        }
        _ => Err(LintError::ArgumentTypeMismatch.at(arg_pos)),
    }
}

fn arg_pos_to_expr_type_and_opt_args(
    arg_pos: &ExpressionPos,
) -> Option<(&ExpressionType, Option<&Expressions>)> {
    match &arg_pos.element {
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

fn has_at_least_one_arg(opt_args: Option<&Expressions>) -> bool {
    match opt_args {
        Some(args) => !args.is_empty(),
        _ => true, // not an array expression
    }
}

impl<'a, R> UserDefinedFunctionLinter<'a, R>
where
    R: HasSubprograms,
{
    fn visit_function(
        &self,
        name: &Name,
        pos: Position,
        args: &Expressions,
    ) -> Result<(), LintErrorPos> {
        if let Name::Qualified(bare_name, qualifier) = name {
            match self.linter_context.functions().get(bare_name) {
                Some(function_signature_pos) => {
                    if function_signature_pos.element != *qualifier {
                        Err(LintError::TypeMismatch.at(function_signature_pos))
                    } else {
                        lint_call_args(args, function_signature_pos.element.param_types(), pos)
                    }
                }
                None => self.handle_undefined_function(args),
            }
        } else {
            panic!("Unresolved function {:?}", name)
        }
    }

    fn handle_undefined_function(&self, args: &Expressions) -> Result<(), LintErrorPos> {
        for i in 0..args.len() {
            let arg_pos = args.get(i).unwrap();
            match arg_pos.expression_type() {
                ExpressionType::BuiltIn(q) => {
                    if q == TypeQualifier::DollarString {
                        return Err(LintError::ArgumentTypeMismatch.at(arg_pos));
                    }
                }
                _ => {
                    return Err(LintError::ArgumentTypeMismatch.at(arg_pos));
                }
            }
        }

        // is converted to a literal 0 in undefined_function_reducer
        Ok(())
    }
}

impl<'a, R> PostConversionLinter for UserDefinedFunctionLinter<'a, R>
where
    R: HasSubprograms,
{
    fn visit_expression(&mut self, expr_pos: &ExpressionPos) -> Result<(), LintErrorPos> {
        let Positioned { element: e, pos } = expr_pos;
        match e {
            Expression::FunctionCall(n, args) => {
                for x in args {
                    self.visit_expression(x)?;
                }
                self.visit_function(n, *pos, args)
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
