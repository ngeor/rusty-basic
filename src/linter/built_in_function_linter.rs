use super::error::*;
use super::post_conversion_linter::PostConversionLinter;
use super::types::*;
use crate::common::*;
use crate::parser::{HasQualifier, NameTrait, QualifiedName, TypeQualifier};
use std::convert::TryFrom;

pub struct BuiltInFunctionLinter;

enum BuiltInFunction {
    /// ENVIRON$
    Environ,
}

impl From<&CaseInsensitiveString> for Option<BuiltInFunction> {
    fn from(s: &CaseInsensitiveString) -> Option<BuiltInFunction> {
        if s == "ENVIRON" {
            Some(BuiltInFunction::Environ)
        } else {
            None
        }
    }
}

impl TryFrom<&QualifiedName> for Option<BuiltInFunction> {
    type Error = Error;
    fn try_from(q: &QualifiedName) -> Result<Option<BuiltInFunction>, Self::Error> {
        let opt_built_in: Option<BuiltInFunction> = q.bare_name().into();
        match opt_built_in {
            Some(b) => match b {
                BuiltInFunction::Environ => {
                    if q.qualifier() == TypeQualifier::DollarString {
                        Ok(Some(b))
                    } else {
                        err_no_pos(LinterError::TypeMismatch)
                    }
                }
            },
            None => Ok(None),
        }
    }
}

pub fn is_built_in_function(function_name: &CaseInsensitiveString) -> bool {
    let opt_built_in: Option<BuiltInFunction> = function_name.into();
    opt_built_in.is_some()
}

impl BuiltInFunctionLinter {
    fn visit_function(
        &self,
        name: &QualifiedName,
        args: &Vec<ExpressionNode>,
    ) -> Result<(), Error> {
        match Option::<BuiltInFunction>::try_from(name)? {
            Some(b) => match b {
                BuiltInFunction::Environ => self.visit_environ(args),
            },
            None => Ok(()),
        }
    }

    fn visit_environ(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
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
