use super::post_conversion_linter::PostConversionLinter;
use super::subprogram_context::FunctionMap;
use super::types::*;
use crate::common::*;
use crate::parser::{BareName, CanCastTo, HasQualifier, QualifiedName, TypeQualifier};

pub struct UserDefinedFunctionLinter<'a> {
    pub functions: &'a FunctionMap,
}

pub fn lint_call_args(
    args: &Vec<ExpressionNode>,
    param_types: &Vec<ResolvedTypeDefinition>,
) -> Result<(), QErrorNode> {
    if args.len() != param_types.len() {
        return err_no_pos(QError::ArgumentCountMismatch);
    }

    for (arg_node, param_type) in args.iter().zip(param_types.iter()) {
        let arg = arg_node.as_ref();
        let arg_q = arg_node.try_type_definition()?;
        match arg {
            Expression::Variable(_) => {
                // it's by ref, it needs to match exactly
                if arg_q != *param_type {
                    return Err(QError::ArgumentTypeMismatch).with_err_at(arg_node);
                }
            }
            _ => {
                // it's by val, casting is allowed
                if !arg_q.can_cast_to(param_type) {
                    return Err(QError::ArgumentTypeMismatch).with_err_at(arg_node);
                }
            }
        }
    }
    Ok(())
}

impl<'a> UserDefinedFunctionLinter<'a> {
    fn visit_function(
        &self,
        name: &QualifiedName,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), QErrorNode> {
        let bare_name: &BareName = name.as_ref();
        match self.functions.get(bare_name) {
            Some((return_type, param_types, _)) => {
                if *return_type != name.qualifier() {
                    err_no_pos(QError::TypeMismatch)
                } else {
                    lint_call_args(args, param_types)
                }
            }
            None => self.handle_undefined_function(args),
        }
    }

    fn handle_undefined_function(&self, args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        for i in 0..args.len() {
            let arg_node = args.get(i).unwrap();
            match arg_node.try_type_definition()? {
                ResolvedTypeDefinition::BuiltIn(q) => {
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
            Expression::BinaryExpression(_, left, right) => {
                self.visit_expression(left)?;
                self.visit_expression(right)
            }
            Expression::UnaryExpression(_, child) => self.visit_expression(child),
            _ => Ok(()),
        }
    }
}
