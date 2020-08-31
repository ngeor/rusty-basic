// ENVIRON str-expr$ -> sets the variable.
// Parameter must be in the form of name=value or name value (TODO support the latter)

use super::BuiltInRun;
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};
use crate::variant::Variant;

pub struct Environ {}

impl BuiltInRun for Environ {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        match interpreter.pop_unnamed_val().unwrap() {
            Variant::VString(arg_string_value) => {
                let parts: Vec<&str> = arg_string_value.split("=").collect();
                if parts.len() != 2 {
                    Err(QError::from("Invalid expression. Must be name=value.")).with_err_no_pos()
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
