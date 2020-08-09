// ENVIRON str-expr$ -> sets the variable.
// Parameter must be in the form of name=value or name value (TODO support the latter)

use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterError, InterpreterErrorNode, Stdlib};
use crate::linter::{ExpressionNode, LinterError, LinterErrorNode};
use crate::parser::TypeQualifier;
use crate::variant::Variant;

pub struct Environ {}

impl BuiltInLint for Environ {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), LinterErrorNode> {
        if args.len() != 1 {
            Err(LinterError::ArgumentCountMismatch).with_err_no_pos()
        } else if args[0].try_qualifier()? != TypeQualifier::DollarString {
            Err(LinterError::ArgumentTypeMismatch).with_err_at(&args[0])
        } else {
            Ok(())
        }
    }
}

impl BuiltInRun for Environ {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
        match interpreter.pop_unnamed_val().unwrap() {
            Variant::VString(arg_string_value) => {
                let parts: Vec<&str> = arg_string_value.split("=").collect();
                if parts.len() != 2 {
                    Err(InterpreterError::from(
                        "Invalid expression. Must be name=value.",
                    ))
                    .with_err_no_pos()
                } else {
                    interpreter
                        .stdlib
                        .set_env_var(parts[0].to_string(), parts[1].to_string());
                    Ok(())
                }
            }
            _ => panic!("Type mismatch"),
        }
    }
}
