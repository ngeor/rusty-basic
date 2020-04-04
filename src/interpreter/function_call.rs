use super::function_context::FunctionImplementation;
use super::*;
use crate::common::Result;
use crate::parser::*;

impl<S: Stdlib> Interpreter<S> {
    pub fn evaluate_function_call(
        &mut self,
        function_name: &QName,
        args: &Vec<Expression>,
    ) -> Result<Variant> {
        let implementation = self._lookup_implementation(function_name)?;
        let arg_values: Vec<Variant> = self._evaluate_arguments(args)?;
        match implementation {
            Some(function_implementation) => {
                self._do_evaluate_function_call(function_implementation, arg_values)
            }
            None => self._handle_undefined_function(function_name, arg_values),
        }
    }

    fn _lookup_implementation(
        &self,
        function_name: &QName,
    ) -> Result<Option<FunctionImplementation>> {
        match function_name {
            QName::Untyped(bare_function_name) => Ok(self
                .function_context
                .get_function_implementation(bare_function_name)),
            QName::Typed(qualified_function_name) => {
                match self
                    .function_context
                    .get_function_implementation(&qualified_function_name.name)
                {
                    Some(function_implementation) => {
                        if function_implementation.name.qualifier
                            != qualified_function_name.qualifier
                        {
                            Err("Duplicate definition".to_string())
                        } else {
                            Ok(Some(function_implementation))
                        }
                    }
                    None => Ok(None),
                }
            }
        }
    }

    fn _evaluate_arguments(&mut self, args: &Vec<Expression>) -> Result<Vec<Variant>> {
        let mut i = 0;
        let mut result: Vec<Variant> = vec![];
        while i < args.len() {
            let variable_value = self.evaluate_expression(&args[i])?;
            result.push(variable_value);
            i += 1;
        }
        Ok(result)
    }

    fn _do_evaluate_function_call(
        &mut self,
        function_implementation: FunctionImplementation,
        args: Vec<Variant>,
    ) -> Result<Variant> {
        let function_parameters: Vec<QualifiedName> = function_implementation.parameters;
        if function_parameters.len() != args.len() {
            self.err(format!(
                "Function {} expected {} parameters but {} were given",
                function_implementation.name,
                function_parameters.len(),
                args.len()
            ))
        } else {
            self.push_context(function_implementation.name.clone());
            self._populate_new_context(function_parameters, args)?;
            self.statements(&function_implementation.block)?;
            let result = self._get_variable_name_or_default(&function_implementation.name);
            self.pop_context();
            Ok(result)
        }
    }

    fn _populate_new_context(
        &mut self,
        mut parameter_names: Vec<QualifiedName>,
        mut arguments: Vec<Variant>,
    ) -> Result<()> {
        while !parameter_names.is_empty() {
            let variable_name = parameter_names.pop().unwrap();
            self.set_variable(variable_name, arguments.pop().unwrap());
        }
        Ok(())
    }

    fn _get_variable_name_or_default(&self, function_name: &QualifiedName) -> Variant {
        match self.get_variable(function_name) {
            Ok(v) => v.clone(),
            Err(_) => _default_variant(&function_name.qualifier),
        }
    }

    fn _handle_undefined_function(
        &self,
        function_name: &QName,
        args: Vec<Variant>,
    ) -> Result<Variant> {
        for arg in args {
            match arg {
                Variant::VString(_) => return self.err("Type mismatch"),
                _ => (),
            }
        }
        Ok(_default_variant(
            &self.effective_type_qualifier(function_name),
        ))
    }
}

fn _default_variant(type_qualifier: &TypeQualifier) -> Variant {
    match type_qualifier {
        TypeQualifier::BangSingle => Variant::VSingle(0.0),
        TypeQualifier::HashDouble => Variant::VDouble(0.0),
        TypeQualifier::DollarString => Variant::VString(String::new()),
        TypeQualifier::PercentInteger => Variant::VInteger(0),
        TypeQualifier::AmpersandLong => Variant::VLong(0),
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

    #[test]
    fn test_function_call_declared_and_implemented() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        X = Add(1, 2)
        FUNCTION Add(A, B)
            Add = A + B
        END FUNCTION
        ";
        let interpreter = interpret(program, MockStdlib::new()).unwrap();
        interpreter.has_variable("X", 3.0_f32);
    }

    #[test]
    fn test_function_call_without_implementation() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        X = Add(1, 2)
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Subprogram not defined"
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
        let interpreter = interpret(program, MockStdlib::new()).unwrap();
        interpreter.has_variable("X", 3.0_f32);
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
        let interpreter = interpret(program, MockStdlib::new()).unwrap();
        interpreter.has_variable("X", 0.0_f32);
        assert_eq!(interpreter.stdlib.output, vec!["3"]);
    }

    #[test]
    fn test_function_call_missing_returns_zero() {
        let program = "
        X = Add(1, 2)
        ";
        let interpreter = interpret(program, MockStdlib::new()).unwrap();
        interpreter.has_variable("X", 0.0_f32);
    }

    #[test]
    fn test_function_call_missing_with_string_arguments_gives_type_mismatch() {
        let program = "
        X = Add(\"1\", \"2\")
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Type mismatch"
        );
    }
}
