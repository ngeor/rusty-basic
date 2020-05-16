use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::common::*;
use crate::parser::TypeQualifier;

pub struct BuiltInFunctionLinter;

impl BuiltInFunctionLinter {
    fn visit_function(
        &self,
        name: &BuiltInFunction,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), Error> {
        match name {
            BuiltInFunction::Environ => self.visit_environ(args),
            BuiltInFunction::Len => self.visit_len(args),
            BuiltInFunction::Str => self.visit_str(args),
            BuiltInFunction::Val => self.visit_val(args),
        }
    }

    fn visit_environ(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        self.require_single_string_argument(args)
    }

    fn visit_len(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 1 {
            err_no_pos(LinterError::ArgumentCountMismatch)
        } else {
            let arg: &Expression = args[0].as_ref();
            match arg {
                Expression::Variable(_) => Ok(()),
                _ => {
                    let q = arg.try_qualifier()?;
                    if q != TypeQualifier::DollarString {
                        err_l(LinterError::VariableRequired, &args[0])
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }

    fn visit_str(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 1 {
            err_no_pos(LinterError::ArgumentCountMismatch)
        } else {
            let q = args[0].as_ref().try_qualifier()?;
            if q == TypeQualifier::DollarString {
                err_l(LinterError::ArgumentTypeMismatch, &args[0])
            } else {
                Ok(())
            }
        }
    }

    fn visit_val(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        self.require_single_string_argument(args)
    }

    fn require_single_string_argument(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 1 {
            err_no_pos(LinterError::ArgumentCountMismatch)
        } else {
            let q = args[0].as_ref().try_qualifier()?;
            if q != TypeQualifier::DollarString {
                err_l(LinterError::ArgumentTypeMismatch, &args[0])
            } else {
                Ok(())
            }
        }
    }
}

impl PostConversionLinter for BuiltInFunctionLinter {
    fn visit_expression(&self, expr_node: &ExpressionNode) -> Result<(), Error> {
        let pos = expr_node.location();
        let e = expr_node.as_ref();
        match e {
            Expression::BuiltInFunctionCall(n, args) => {
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
