use crate::common::Result;
use crate::parser::{Block, QualifiedName};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: QualifiedName,
    pub parameters: Vec<QualifiedName>,
}

impl FunctionDeclaration {
    pub fn new(name: QualifiedName, parameters: Vec<QualifiedName>) -> FunctionDeclaration {
        FunctionDeclaration { name, parameters }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionImplementation {
    pub name: QualifiedName,
    pub parameters: Vec<QualifiedName>,
    pub block: Block,
}

impl FunctionImplementation {
    pub fn new(
        name: QualifiedName,
        parameters: Vec<QualifiedName>,
        block: Block,
    ) -> FunctionImplementation {
        FunctionImplementation {
            name,
            parameters,
            block,
        }
    }
}

/// A function context
#[derive(Debug)]
pub struct FunctionContext {
    function_declaration_map: HashMap<String, FunctionDeclaration>,
    function_implementation_map: HashMap<String, FunctionImplementation>,
}

impl FunctionContext {
    pub fn new() -> FunctionContext {
        FunctionContext {
            function_declaration_map: HashMap::new(),
            function_implementation_map: HashMap::new(),
        }
    }

    pub fn add_function_declaration(
        &mut self,
        function_name: QualifiedName,
        parameters: Vec<QualifiedName>,
    ) -> Result<()> {
        if self
            .function_declaration_map
            .contains_key(&function_name.name)
        {
            let existing_declaration = self
                .function_declaration_map
                .get(&function_name.name)
                .unwrap();
            if existing_declaration.name != function_name {
                Err("Duplicate definition".to_string())
            } else {
                if existing_declaration.parameters.len() != parameters.len() {
                    Err("Argument-count mismatch".to_string())
                } else {
                    if existing_declaration.parameters == parameters {
                        Ok(())
                    } else {
                        Err("Parameter type mismatch".to_string())
                    }
                }
            }
        } else {
            self.function_declaration_map.insert(
                function_name.name.clone(),
                FunctionDeclaration::new(function_name, parameters),
            );
            Ok(())
        }
    }

    pub fn add_function_implementation(
        &mut self,
        function_name: QualifiedName,
        parameters: Vec<QualifiedName>,
        block: Block,
    ) -> Result<()> {
        if self
            .function_implementation_map
            .contains_key(&function_name.name)
        {
            Err("Duplicate definition".to_string())
        } else {
            if self
                .function_declaration_map
                .contains_key(&function_name.name)
            {
                let existing_declaration = self
                    .function_declaration_map
                    .get(&function_name.name)
                    .unwrap();
                if existing_declaration.name != function_name {
                    return Err("Duplicate definition".to_string());
                }

                if existing_declaration.parameters.len() != parameters.len() {
                    return Err("Argument-count mismatch".to_string());
                }

                if existing_declaration.parameters != parameters {
                    return Err("Parameter type mismatch".to_string());
                }
            }

            self.function_implementation_map.insert(
                function_name.name.clone(),
                FunctionImplementation::new(function_name, parameters, block),
            );
            Ok(())
        }
    }

    pub fn get_function_declarations(
        &self,
    ) -> std::collections::hash_map::Keys<String, FunctionDeclaration> {
        self.function_declaration_map.keys()
    }

    pub fn get_function_implementation(&self, name: &String) -> Option<FunctionImplementation> {
        self.function_implementation_map
            .get(name)
            .map(|x| x.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

    #[test]
    fn test_duplicate_function_declaration_identical_is_tolerated() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        DECLARE FUNCTION Add(A, B)
        PRINT Add(1, 2)
        FUNCTION Add(A, B)
        Add = A + B
        END FUNCTION
        ";
        let interpreter = interpret(program, MockStdlib::new()).unwrap();
        assert_eq!(interpreter.stdlib.output, vec!["3"]);
    }

    #[test]
    fn test_duplicate_function_same_type_different_argument_count() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        DECLARE FUNCTION Add(A, B, C)
        PRINT Add(1, 2)
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Argument-count mismatch"
        );
    }

    #[test]
    fn test_declaration_implementation_different_argument_count() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        PRINT Add(1, 2)
        FUNCTION Add(A, B, C)
            Add = A + B +C
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Argument-count mismatch"
        );
    }

    #[test]
    fn test_duplicate_function_different_function_type() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        DECLARE FUNCTION Add%(A, B)
        PRINT Add(1, 2)
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Duplicate definition"
        );
    }

    #[test]
    fn test_duplicate_function_implementation() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        PRINT Add(1, 2)
        FUNCTION Add(A, B)
        Add = A + B
        END FUNCTION
        FUNCTION Add(A, B)
        Add = A + B
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Duplicate definition"
        );
    }

    #[test]
    fn test_duplicate_function_different_parameter_type() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        DECLARE FUNCTION Add(A$, B)
        PRINT Add(1, 2)
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Parameter type mismatch"
        );
    }

    #[test]
    fn test_declaration_implementation_different_parameter_type() {
        let program = "
        DECLARE FUNCTION Add(A, B)
        PRINT Add(1, 2)
        FUNCTION Add(A, B$)
        Add = A + B
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Parameter type mismatch"
        );
    }

    #[test]
    fn test_duplicate_definition_on_call() {
        let program = "
        DECLARE FUNCTION Add#(A, B)
        PRINT Add!(1, 2)
        FUNCTION Add#(A, B)
            Add# = A + B
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Duplicate definition"
        );
    }

    #[test]
    fn test_duplicate_definition_on_implementation() {
        let program = "
        DECLARE FUNCTION Add#(A, B)
        PRINT Add#(1, 2)
        FUNCTION Add(A, B)
            Add = A + B
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Duplicate definition"
        );
    }

    #[test]
    fn test_duplicate_definition_on_return_value() {
        let program = "
        DECLARE FUNCTION Add#(A, B)
        PRINT Add#(1, 2)
        FUNCTION Add#(A, B)
            Add! = A + B
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap_err(),
            "Duplicate definition"
        );
    }

    #[test]
    fn test_able_to_call_function_with_type_qualifier() {
        let program = "
        DECLARE FUNCTION Add#(A, B)
        PRINT Add#(1, 2)
        FUNCTION Add#(A, B)
            Add# = A + B
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap().stdlib.output,
            vec!["3"]
        );
    }

    #[test]
    fn test_able_to_call_function_without_type_qualifier() {
        let program = "
        DECLARE FUNCTION Add#(A, B)
        PRINT Add(1, 2)
        FUNCTION Add#(A, B)
            Add# = A + B
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap().stdlib.output,
            vec!["3"]
        );
    }

    #[test]
    fn test_able_to_return_value_without_type_qualifier() {
        let program = "
        DECLARE FUNCTION Add#(A, B)
        PRINT Add#(1, 2)
        FUNCTION Add#(A, B)
            Add = A + B
        END FUNCTION
        ";
        assert_eq!(
            interpret(program, MockStdlib::new()).unwrap().stdlib.output,
            vec!["3"]
        );
    }
}
