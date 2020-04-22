use crate::common::CaseInsensitiveString;
use crate::interpreter::subprogram_context::{
    CmpQualifier, QualifiedImplementationNode, SubprogramContext,
};
use crate::parser::{BareNameNode, ResolveInto, TypeResolver};

//pub type QualifiedSubDeclarationNode = QualifiedDeclarationNode<CaseInsensitiveString>;
pub type QualifiedSubImplementationNode = QualifiedImplementationNode<CaseInsensitiveString>;
pub type SubContext = SubprogramContext<CaseInsensitiveString>;

impl ResolveInto<CaseInsensitiveString> for BareNameNode {
    fn resolve_into<T: TypeResolver>(self, _resolver: &T) -> CaseInsensitiveString {
        self.into()
    }
}

impl CmpQualifier<CaseInsensitiveString> for BareNameNode {
    fn eq_qualifier<TR: TypeResolver>(
        _left: &Self,
        _right: &CaseInsensitiveString,
        _resolver: &TR,
    ) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::Location;
    use crate::interpreter::InterpreterError;

    #[test]
    fn test_duplicate_sub_declaration_identical_is_tolerated() {
        let program = "
        DECLARE SUB Add(A, B)
        DECLARE SUB Add(A, B)
        Add 1, 2
        SUB Add(A, B)
            PRINT A + B
        END SUB
        ";
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.output, vec!["3"]);
    }

    #[test]
    fn test_duplicate_sub_same_type_different_argument_count() {
        let program = "
        DECLARE SUB Add(A, B)
        DECLARE SUB Add(A, B, C)
        Add 1, 2
        ";
        assert_eq!(
            interpret_err(program),
            InterpreterError::new_with_pos("Argument-count mismatch", Location::new(3, 9))
        );
    }

    #[test]
    fn test_declaration_implementation_different_argument_count() {
        let program = "
        DECLARE SUB Add(A, B)
        Add 1, 2
        SUB Add(A, B, C)
            PRINT A + B +C
        END SUB
        ";
        assert_eq!(
            interpret_err(program),
            InterpreterError::new_with_pos("Argument-count mismatch", Location::new(4, 9))
        );
    }

    #[test]
    fn test_duplicate_sub_implementation() {
        let program = "
        DECLARE SUB Add(A, B)
        Add 1, 2
        SUB Add(A, B)
            PRINT A + B
        END SUB
        SUB Add(A, B)
            PRINT A + B
        END SUB
        ";
        assert_eq!(
            interpret_err(program),
            InterpreterError::new_with_pos("Duplicate definition", Location::new(7, 9))
        );
    }

    #[test]
    fn test_duplicate_sub_different_parameter_type() {
        let program = "
        DECLARE SUB Add(A, B)
        DECLARE SUB Add(A$, B)
        Add 1, 2
        ";
        assert_eq!(
            interpret_err(program),
            InterpreterError::new_with_pos("Parameter type mismatch", Location::new(3, 25))
        );
    }

    #[test]
    fn test_sub_declaration_implementation_different_parameter_type() {
        let program = "
        DECLARE SUB Add(A, B)
        Add 1, 2
        SUB Add(A, B$)
            PRINT A
            PRINT B$
        END SUB
        ";
        assert_eq!(
            interpret_err(program),
            InterpreterError::new_with_pos("Parameter type mismatch", Location::new(4, 20))
        );
    }
}
