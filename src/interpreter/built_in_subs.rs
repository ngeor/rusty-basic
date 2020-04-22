use crate::common::HasLocation;
use crate::interpreter::{InterpreterError, Result, Stdlib, Variant};
use crate::parser::{BareNameNode, ExpressionNode};

pub fn supports_sub(sub_name: &BareNameNode) -> bool {
    sub_name.as_ref() == "ENVIRON"
}

pub fn call_sub<S: Stdlib>(
    interpreter: &mut S,
    sub_name: &BareNameNode,
    args: &Vec<ExpressionNode>,
    arg_values: Vec<Variant>,
) -> Result<()> {
    if sub_name.as_ref() == "ENVIRON" {
        _do_environ_sub(interpreter, sub_name, args, arg_values)
    } else {
        panic!("should not have been called");
    }
}

fn _do_environ_sub<S: Stdlib>(
    interpreter: &mut S,
    sub_name_node: &BareNameNode,
    args: &Vec<ExpressionNode>,
    arg_values: Vec<Variant>,
) -> Result<()> {
    if args.len() != 1 {
        return Err(InterpreterError::new_with_pos(
            "ENVIRON requires exactly 1 argument",
            sub_name_node.location(),
        ));
    }

    match &arg_values[0] {
        Variant::VString(arg_string_value) => {
            let parts: Vec<&str> = arg_string_value.split("=").collect();
            if parts.len() != 2 {
                Err(InterpreterError::new_with_pos(
                    "Invalid expression. Must be name=value.",
                    args[0].location(),
                ))
            } else {
                interpreter.set_env_var(parts[0].to_string(), parts[1].to_string());
                Ok(())
            }
        }
        _ => Err(InterpreterError::new_with_pos(
            "Type mismatch",
            args[0].location(),
        )),
    }
}
