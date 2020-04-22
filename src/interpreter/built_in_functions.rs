use crate::common::HasLocation;
use crate::interpreter::{InterpreterError, Result, Stdlib, Variant};
use crate::parser::{ExpressionNode, Name, NameNode};

fn _do_environ_function<S: Stdlib>(
    stdlib: &S,
    function_name: &NameNode,
    args: &Vec<ExpressionNode>,
    arg_values: Vec<Variant>,
) -> Result<Variant> {
    if arg_values.len() != 1 {
        Err(InterpreterError::new_with_pos(
            "ENVIRON$ expected exactly one argument",
            function_name.location(),
        ))
    } else {
        match &arg_values[0] {
            Variant::VString(env_var_name) => {
                Ok(Variant::VString(stdlib.get_env_var(env_var_name)))
            }
            _ => Err(InterpreterError::new_with_pos(
                "Type mismatch at ENVIRON$",
                args[0].location(),
            )),
        }
    }
}

pub fn supports_function(function_name: &NameNode) -> bool {
    function_name == &Name::from("ENVIRON$")
}

pub fn call_function<S: Stdlib>(
    stdlib: &S,
    function_name: &NameNode,
    args: &Vec<ExpressionNode>,
    arg_values: Vec<Variant>,
) -> Result<Variant> {
    if function_name == &Name::from("ENVIRON$") {
        _do_environ_function(stdlib, function_name, args, arg_values)
    } else {
        panic!("should not have been called");
    }
}
