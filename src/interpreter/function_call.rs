use super::{Interpreter, Result, Stdlib, Variant};
use crate::parser::{ExpressionNode, NameNode};

mod built_in_function {
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

    pub fn supports(function_name: &NameNode) -> bool {
        function_name == &Name::from("ENVIRON$")
    }

    pub fn call<S: Stdlib>(
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
}

mod undefined_function {
    use crate::common::HasLocation;
    use crate::interpreter::{InterpreterError, Result, Variant};
    use crate::parser::{ExpressionNode, HasBareName, NameNode, TypeResolver};

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
            resolver.resolve(function_name.bare_name()),
        ))
    }
}

mod user_function {
    use crate::common::{HasLocation, Location};
    use crate::interpreter::function_context::LookupImplementation;
    use crate::interpreter::statement::StatementRunner;
    use crate::interpreter::variable_getter::VariableGetter;
    use crate::interpreter::variable_setter::VariableSetter;
    use crate::interpreter::{InterpreterError, PushPopContext, Result, Variant};
    use crate::parser::{BlockNode, ExpressionNode, HasQualifier, Name, NameNode, QualifiedName};

    fn _populate_new_context<VS: VariableSetter<NameNode>>(
        variable_setter: &mut VS,
        mut parameter_names: Vec<QualifiedName>,
        mut arguments: Vec<Variant>,
        call_pos: Location,
    ) -> Result<()> {
        while !parameter_names.is_empty() {
            let variable_name = parameter_names.pop().unwrap();
            let name_node = NameNode::new(Name::Typed(variable_name), call_pos);
            variable_setter.set_variable(name_node, arguments.pop().unwrap())?;
        }
        Ok(())
    }

    fn _get_variable_name_or_default<VG: VariableGetter>(
        variable_getter: &VG,
        function_name: &QualifiedName,
        pos: Location,
    ) -> Variant {
        match variable_getter.get_variable_at(function_name, pos) {
            Ok(v) => v.clone(),
            Err(_) => Variant::default_variant(function_name.qualifier()),
        }
    }

    pub fn supports<LI: LookupImplementation>(
        function_context: &LI,
        function_name: &NameNode,
    ) -> Result<bool> {
        function_context
            .lookup_implementation(function_name)
            .map(|x| x.is_some())
    }

    pub fn call<TI>(
        interpreter: &mut TI,
        function_name: &NameNode,
        args: &Vec<ExpressionNode>,
        arg_values: Vec<Variant>,
    ) -> Result<Variant>
    where
        TI: VariableGetter
            + VariableSetter<NameNode>
            + PushPopContext
            + StatementRunner<BlockNode>
            + LookupImplementation,
    {
        let function_implementation = interpreter.lookup_implementation(function_name)?.unwrap();
        let function_parameters: Vec<QualifiedName> = function_implementation.parameters;
        if function_parameters.len() != args.len() {
            Err(InterpreterError::new_with_pos(
                format!(
                    "Function {} expected {} parameters but {} were given",
                    function_implementation.name,
                    function_parameters.len(),
                    args.len()
                ),
                function_name.location(),
            ))
        } else {
            interpreter.push_context(function_implementation.name.clone());
            _populate_new_context(
                interpreter,
                function_parameters,
                arg_values,
                function_name.location(),
            )?;
            interpreter
                .run(&function_implementation.block)
                .map_err(|e| e.merge_pos(function_name.location()))?;
            let result = _get_variable_name_or_default(
                interpreter,
                &function_implementation.name,
                function_name.location(),
            );
            interpreter.pop_context();
            Ok(result)
        }
    }
}

impl<S: Stdlib> Interpreter<S> {
    pub fn evaluate_function_call(
        &mut self,
        function_name: &NameNode,
        args: &Vec<ExpressionNode>,
    ) -> Result<Variant> {
        let arg_values: Vec<Variant> = self._evaluate_arguments(args)?;

        if built_in_function::supports(function_name) {
            built_in_function::call(&self.stdlib, function_name, args, arg_values)
        } else {
            if user_function::supports(self, function_name)? {
                user_function::call(self, function_name, args, arg_values)
            } else {
                undefined_function::call(self, function_name, args, arg_values)
            }
        }
    }

    fn _evaluate_arguments(&mut self, args: &Vec<ExpressionNode>) -> Result<Vec<Variant>> {
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
