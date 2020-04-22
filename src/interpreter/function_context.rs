use crate::interpreter::subprogram_context::{
    CmpQualifier, QualifiedImplementationNode, SubprogramContext,
};
use crate::parser::{
    HasQualifier, NameNode, QualifiedName, ResolveIntoRef, TypeQualifier, TypeResolver,
};

//pub type QualifiedFunctionDeclarationNode = QualifiedDeclarationNode<QualifiedName>;
pub type QualifiedFunctionImplementationNode = QualifiedImplementationNode<QualifiedName>;
pub type FunctionContext = SubprogramContext<QualifiedName>;

impl CmpQualifier<QualifiedName> for NameNode {
    fn eq_qualifier<TR: TypeResolver>(left: &Self, right: &QualifiedName, resolver: &TR) -> bool {
        let left_qualifier: TypeQualifier = left.resolve_into(resolver);
        left_qualifier == right.qualifier()
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::Location;
    use crate::interpreter::InterpreterError;

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
        let interpreter = interpret(program);
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
            interpret_err(program),
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
            interpret_err(program),
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
            interpret_err(program),
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
            interpret_err(program),
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
            interpret_err(program),
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
            interpret_err(program),
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
            interpret_err(program),
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
            interpret_err(program),
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
            interpret_err(program),
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
        assert_eq!(interpret(program).stdlib.output, vec!["3"]);
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
        assert_eq!(interpret(program).stdlib.output, vec!["3"]);
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
        assert_eq!(interpret(program).stdlib.output, vec!["3"]);
    }
}
