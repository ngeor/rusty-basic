use super::function_context::QualifiedFunctionImplementationNode;
use super::variable_setter::VariableSetter;
use super::{Interpreter, InterpreterError, Result, Stdlib, VariableGetter, Variant};
use crate::common::{HasLocation, Location};
use crate::parser::{
    ExpressionNode, HasBareName, HasQualifier, NameNode, QualifiedName, TypeResolver,
};

impl<S: Stdlib> Interpreter<S> {
    pub fn evaluate_function_call(
        &mut self,
        function_name: &NameNode,
        args: &Vec<ExpressionNode>,
    ) -> Result<Variant> {
        let implementation = self.function_context.lookup_implementation(function_name)?;
        let arg_values: Vec<Variant> = self._evaluate_arguments(args)?;
        match implementation {
            Some(function_implementation) => {
                self._do_evaluate_function_call(function_implementation, arg_values, function_name)
            }
            None => self._handle_undefined_function(function_name, arg_values, args),
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

    fn _do_evaluate_function_call(
        &mut self,
        function_implementation: QualifiedFunctionImplementationNode,
        args: Vec<Variant>,
        function_name: &NameNode,
    ) -> Result<Variant> {
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
            self.push_context(function_implementation.name.clone());
            self._populate_new_context(function_parameters, args, function_name.location())?;
            self.statements(&function_implementation.block)
                .map_err(|e| e.merge_pos(function_name.location()))?;
            let result = self._get_variable_name_or_default(
                &function_implementation.name,
                function_name.location(),
            );
            self.pop_context();
            Ok(result)
        }
    }

    fn _populate_new_context(
        &mut self,
        mut parameter_names: Vec<QualifiedName>,
        mut arguments: Vec<Variant>,
        call_pos: Location,
    ) -> Result<()> {
        while !parameter_names.is_empty() {
            let variable_name = parameter_names.pop().unwrap();
            self.set_variable((variable_name, call_pos), arguments.pop().unwrap())?;
        }
        Ok(())
    }

    fn _get_variable_name_or_default(
        &self,
        function_name: &QualifiedName,
        pos: Location,
    ) -> Variant {
        match self.get_variable((function_name, pos)) {
            Ok(v) => v.clone(),
            Err(_) => Variant::default_variant(function_name.qualifier()),
        }
    }

    fn _handle_undefined_function(
        &self,
        function_name: &NameNode,
        arg_values: Vec<Variant>,
        arg_nodes: &Vec<ExpressionNode>,
    ) -> Result<Variant> {
        for i in 0..arg_values.len() {
            let arg_value = &arg_values[i];
            let arg_node = &arg_nodes[i];
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
            self.resolve(function_name.bare_name()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::assert_has_variable;
    use crate::common::Location;

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
}
