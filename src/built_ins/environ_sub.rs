// ENVIRON str-expr$ -> sets the variable.
// Parameter must be in the form of name=value or name value (TODO support the latter)

use super::{BuiltInLint, BuiltInRun};
use crate::common::Location;
use crate::interpreter::{err, Interpreter, InterpreterError, Stdlib};
use crate::linter::{err_l, err_no_pos, Error, ExpressionNode, LinterError};
use crate::parser::TypeQualifier;
use crate::variant::Variant;

pub struct Environ {}

impl BuiltInLint for Environ {
    fn lint(&self, args: &Vec<ExpressionNode>) -> Result<(), Error> {
        if args.len() != 1 {
            err_no_pos(LinterError::ArgumentCountMismatch)
        } else if args[0].try_qualifier()? != TypeQualifier::DollarString {
            err_l(LinterError::ArgumentTypeMismatch, &args[0])
        } else {
            Ok(())
        }
    }
}

impl BuiltInRun for Environ {
    fn run<S: Stdlib>(
        &self,
        interpreter: &mut Interpreter<S>,
        pos: Location,
    ) -> Result<(), InterpreterError> {
        match interpreter.pop_unnamed_val().unwrap() {
            Variant::VString(arg_string_value) => {
                let parts: Vec<&str> = arg_string_value.split("=").collect();
                if parts.len() != 2 {
                    err("Invalid expression. Must be name=value.", pos)
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
