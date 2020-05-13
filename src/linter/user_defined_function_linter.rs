use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::subprogram_context::FunctionMap;
use super::types::*;
use crate::common::*;
use crate::parser::{HasQualifier, NameTrait, QualifiedName, TypeQualifier};

pub struct UserDefinedFunctionLinter<'a> {
    pub functions: &'a FunctionMap,
}

type ExpressionNodes = Vec<ExpressionNode>;
type TypeQualifiers = Vec<TypeQualifier>;
type Result = std::result::Result<(), Error>;

pub fn lint_call_args(args: &ExpressionNodes, param_types: &TypeQualifiers) -> Result {
    if args.len() != param_types.len() {
        return err_no_pos(LinterError::ArgumentCountMismatch);
    }

    for (arg_node, param_type) in args.iter().zip(param_types.iter()) {
        let arg = arg_node.as_ref();
        let arg_q = arg.try_qualifier()?;
        match arg {
            Expression::Variable(_) => {
                // it's by ref, it needs to match exactly
                if arg_q != *param_type {
                    return err_l(LinterError::ArgumentTypeMismatch, arg_node);
                }
            }
            _ => {
                // it's by val, casting is allowed
                if !arg_q.can_cast_to(*param_type) {
                    return err_l(LinterError::ArgumentTypeMismatch, arg_node);
                }
            }
        }
    }
    Ok(())
}

impl<'a> UserDefinedFunctionLinter<'a> {
    fn visit_function(&self, name: &QualifiedName, args: &Vec<ExpressionNode>) -> Result {
        let bare_name = name.bare_name();
        match self.functions.get(bare_name) {
            Some((return_type, param_types, _)) => {
                if *return_type != name.qualifier() {
                    err_no_pos(LinterError::TypeMismatch)
                } else {
                    lint_call_args(args, param_types)
                }
            }
            None => self.handle_undefined_function(args),
        }
    }

    fn handle_undefined_function(&self, args: &Vec<ExpressionNode>) -> Result {
        for i in 0..args.len() {
            let arg_node = args.get(i).unwrap();
            let arg = arg_node.as_ref();
            let arg_q = arg.try_qualifier()?;
            if arg_q == TypeQualifier::DollarString {
                return err_l(LinterError::ArgumentTypeMismatch, arg_node);
            }
        }

        // is converted to a literal 0 in undefined_function_reducer
        Ok(())
    }
}

impl<'a> PostConversionLinter for UserDefinedFunctionLinter<'a> {
    fn visit_expression(&self, expr_node: &ExpressionNode) -> Result {
        let pos = expr_node.location();
        let e = expr_node.as_ref();
        match e {
            Expression::FunctionCall(n, args) => {
                for x in args {
                    self.visit_expression(x)?;
                }
                self.visit_function(n, args).with_err_pos(pos)
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
