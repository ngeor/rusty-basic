use super::{Interpreter, Result, Stdlib, Variant};
use crate::interpreter::built_in_functions;
use crate::interpreter::user_defined_function;
use crate::parser::{ExpressionNode, NameNode};

mod undefined_function {
    use crate::common::HasLocation;
    use crate::interpreter::{InterpreterError, Result, Variant};
    use crate::parser::{ExpressionNode, NameNode, TypeResolver};

    pub fn call<TR: TypeResolver>(
        resolver: &mut TR,
        function_name: &NameNode,
        args: &Vec<ExpressionNode>,
        arg_values: Vec<Variant>,
    ) -> Result<Variant> {
        for i in 0..arg_values.len() {
            let arg_value = &arg_values[i];
            let arg_node = &args[i];
            match arg_value {
                Variant::VString(_) => {
                    return Err(InterpreterError::new_with_pos(
                        "Type mismatch",
                        arg_node.location(),
                    ))
                }
                _ => (),
            }
        }
        Ok(Variant::default_variant(
            resolver.resolve(function_name.as_ref()),
        ))
    }
}

impl<S: Stdlib> Interpreter<S> {
    pub fn evaluate_function_call(
        &mut self,
        function_name: &NameNode,
        args: &Vec<ExpressionNode>,
    ) -> Result<Variant> {
        let arg_values: Vec<Variant> = self.evaluate_arguments(args)?;

        if built_in_functions::supports_function(function_name) {
            built_in_functions::call_function(&self.stdlib, function_name, args, arg_values)
        } else {
            if user_defined_function::supports_function(self, function_name) {
                user_defined_function::call_function(self, function_name, args, arg_values)
            } else {
                undefined_function::call(self, function_name, args, arg_values)
            }
        }
    }

    pub fn evaluate_arguments(&mut self, args: &Vec<ExpressionNode>) -> Result<Vec<Variant>> {
        let mut result: Vec<Variant> = vec![];
        for arg in args.iter() {
            let variable_value = self.evaluate_expression(arg)?;
            result.push(variable_value);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_has_variable;
    use crate::common::Location;
    use crate::interpreter::{InterpreterError, Stdlib, Variant};

    #[test]
    fn test_function_call_declared_and_implemented() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        X = Add(1, 2)
        FUNCTION Add(A, B)
            Add = A + B
        END FUNCTION
        ";
        let interpreter = interpret(program);
        assert_has_variable!(interpreter, "X", 3.0_f32);
    }

    #[test]
    fn test_function_call_without_implementation() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        X = Add(1, 2)
        ";
        assert_eq!(
            interpret_err(program),
            InterpreterError::new_with_pos("Subprogram not defined", Location::new(2, 9))
        );
    }

    #[test]
    fn test_function_call_without_declaration() {
        let program = "
        X = Add(1, 2)
        FUNCTION Add(A, B)
            Add = A + B
        END FUNCTION
        ";
        let interpreter = interpret(program);
        assert_has_variable!(interpreter, "X", 3.0_f32);
    }

    #[test]
    fn test_function_call_not_setting_return_value_defaults_to_zero() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        X = Add(1, 2)
        FUNCTION Add(A, B)
            PRINT A + B
        END FUNCTION
        ";
        let interpreter = interpret(program);
        assert_has_variable!(interpreter, "X", 0.0_f32);
        assert_eq!(interpreter.stdlib.output, vec!["3"]);
    }

    #[test]
    fn test_function_call_missing_returns_zero() {
        let program = "
        X = Add(1, 2)
        ";
        let interpreter = interpret(program);
        assert_has_variable!(interpreter, "X", 0.0_f32);
    }

    #[test]
    fn test_function_call_missing_with_string_arguments_gives_type_mismatch() {
        let program = "
        X = Add(\"1\", \"2\")
        ";
        assert_eq!(
            interpret_err(program),
            InterpreterError::new_with_pos("Type mismatch", Location::new(2, 17))
        );
    }

    #[test]
    fn test_function_call_lowercase() {
        let program = "
        DECLARE FUNCTION Add(A, B, c)
        X = add(1, 2, 3)
        FUNCTION ADD(a, B, C)
            aDd = a + b + c
        END FUNCTION
        ";
        let interpreter = interpret(program);
        assert_has_variable!(interpreter, "X", 6.0_f32);
    }

    #[test]
    fn test_function_call_defint() {
        let program = "
        DEFINT A-Z
        DECLARE FUNCTION Add(A, B, c)
        X = add(1, 2, 3)
        FUNCTION ADD(a, B, C)
            aDd = a + b + c
        END FUNCTION
        ";
        let interpreter = interpret(program);
        assert_has_variable!(interpreter, "X", 6);
    }

    #[test]
    fn test_function_call_defstr() {
        let program = r#"
        DEFSTR A-Z
        DECLARE FUNCTION Add(A, B, c)
        X = add("1", "2", "3")
        FUNCTION ADD(a, B, C)
            aDd = a + b + c
        END FUNCTION
        "#;
        let interpreter = interpret(program);
        assert_has_variable!(interpreter, "X", "123");
    }

    #[test]
    fn test_function_call_environ() {
        let program = r#"
        X$ = ENVIRON$("abc")
        Y$ = ENVIRON$("def")
        "#;
        let mut stdlib = MockStdlib::new();
        stdlib.set_env_var("abc".to_string(), "foo".to_string());
        let interpreter = interpret_with_stdlib(program, stdlib);
        assert_has_variable!(interpreter, "X$", "foo");
        assert_has_variable!(interpreter, "Y$", "");
    }
}
