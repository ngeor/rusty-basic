use super::{InterpreterError, Result};
use crate::common::{CaseInsensitiveString, HasLocation, Location};
use crate::parser::{
    HasBareName, HasQualifier, NameNode, QualifiedFunctionDeclarationNode,
    QualifiedFunctionImplementationNode, QualifiedNameNode,
};
use std::collections::HashMap;

/// A function context
#[derive(Debug)]
pub struct FunctionContext {
    function_declaration_map: HashMap<CaseInsensitiveString, QualifiedFunctionDeclarationNode>,
    function_implementation_map:
        HashMap<CaseInsensitiveString, QualifiedFunctionImplementationNode>,
}

impl FunctionContext {
    pub fn new() -> FunctionContext {
        FunctionContext {
            function_declaration_map: HashMap::new(),
            function_implementation_map: HashMap::new(),
        }
    }

    pub fn add_function_declaration(&mut self, f: QualifiedFunctionDeclarationNode) -> Result<()> {
        match self._validate_against_existing_declaration(&f.name, &f.parameters, f.pos)? {
            None => {
                self.function_declaration_map
                    .insert(f.bare_name().clone(), f);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn add_function_implementation(
        &mut self,
        f: QualifiedFunctionImplementationNode,
    ) -> Result<()> {
        if self._contains_implementation(&f.name) {
            Err(InterpreterError::new_with_pos(
                "Duplicate definition",
                f.pos,
            ))
        } else {
            self._validate_against_existing_declaration(&f.name, &f.parameters, f.pos)?;
            self.function_implementation_map
                .insert(f.bare_name().clone(), f);
            Ok(())
        }
    }

    fn _validate_against_existing_declaration(
        &self,
        function_name: &QualifiedNameNode,
        parameters: &Vec<QualifiedNameNode>,
        pos: Location,
    ) -> Result<Option<&QualifiedFunctionDeclarationNode>> {
        match self.function_declaration_map.get(function_name.bare_name()) {
            Some(existing_declaration) => {
                if existing_declaration.name.qualifier() != function_name.qualifier() {
                    Err(InterpreterError::new_with_pos("Duplicate definition", pos))
                } else {
                    _are_parameters_same(&existing_declaration.parameters, &parameters, pos)?;
                    Ok(Some(existing_declaration))
                }
            }
            None => Ok(None),
        }
    }

    pub fn get_function_declarations(
        &self,
    ) -> std::collections::hash_map::Keys<CaseInsensitiveString, QualifiedFunctionDeclarationNode>
    {
        self.function_declaration_map.keys()
    }

    pub fn get_function_declaration_pos(&self, name: &CaseInsensitiveString) -> Option<Location> {
        self.function_declaration_map.get(name).map(|x| x.pos)
    }

    pub fn get_function_implementation(
        &self,
        name: &CaseInsensitiveString,
    ) -> Option<QualifiedFunctionImplementationNode> {
        self.function_implementation_map
            .get(name)
            .map(|x| x.clone())
    }

    fn _contains_declaration(&self, function_name: &QualifiedNameNode) -> bool {
        self.function_declaration_map
            .contains_key(function_name.bare_name())
    }

    fn _contains_implementation(&self, function_name: &QualifiedNameNode) -> bool {
        self.function_implementation_map
            .contains_key(function_name.bare_name())
    }

    fn _get_declaration(
        &self,
        function_name: &QualifiedNameNode,
    ) -> Result<&QualifiedFunctionDeclarationNode> {
        match self.function_declaration_map.get(function_name.bare_name()) {
            Some(x) => Ok(x),
            None => Err(InterpreterError::new_with_pos(
                format!("Function {} not declared", function_name.bare_name()),
                function_name.location(),
            )),
        }
    }

    pub fn lookup_implementation(
        &self,
        function_name: &NameNode,
    ) -> Result<Option<QualifiedFunctionImplementationNode>> {
        match function_name {
            NameNode::Bare(bare_function_name) => {
                Ok(self.get_function_implementation(bare_function_name.element()))
            }
            NameNode::Typed(qualified_function_name) => {
                self._lookup_implementation_qualified(qualified_function_name)
            }
        }
    }

    fn _lookup_implementation_qualified(
        &self,
        function_name: &QualifiedNameNode,
    ) -> Result<Option<QualifiedFunctionImplementationNode>> {
        match self.get_function_implementation(function_name.bare_name()) {
            Some(function_implementation) => {
                if function_implementation.name.qualifier() != function_name.qualifier() {
                    // the function is defined as A#
                    // but is being called as A!
                    Err(InterpreterError::new_with_pos(
                        "Duplicate definition",
                        function_name.location(),
                    ))
                } else {
                    Ok(Some(function_implementation))
                }
            }
            None => Ok(None),
        }
    }
}

fn _are_parameters_same(
    existing: &Vec<QualifiedNameNode>,
    parameters: &Vec<QualifiedNameNode>,
    pos: Location,
) -> Result<()> {
    if existing.len() != parameters.len() {
        return Err(InterpreterError::new_with_pos(
            "Argument-count mismatch",
            pos,
        ));
    }

    for i in 0..existing.len() {
        let e = &existing[i];
        let n = &parameters[i];
        if e.qualifier() != n.qualifier() {
            return Err(InterpreterError::new_with_pos(
                "Parameter type mismatch",
                n.location(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::InterpreterError;
    use crate::common::Location;

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
            InterpreterError::new_with_pos("Argument-count mismatch", Location::new(3, 9))
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
            InterpreterError::new_with_pos("Argument-count mismatch", Location::new(4, 9))
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
            InterpreterError::new_with_pos("Duplicate definition", Location::new(3, 9))
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
            InterpreterError::new_with_pos("Duplicate definition", Location::new(7, 9))
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
            InterpreterError::new_with_pos("Parameter type mismatch", Location::new(3, 30))
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
            InterpreterError::new_with_pos("Parameter type mismatch", Location::new(4, 25))
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
            InterpreterError::new_with_pos("Duplicate definition", Location::new(3, 15))
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
            InterpreterError::new_with_pos("Duplicate definition", Location::new(4, 9))
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
            InterpreterError::new(
                "Duplicate definition",
                vec![Location::new(5, 13), Location::new(3, 15)]
            )
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
