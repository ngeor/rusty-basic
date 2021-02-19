use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn test_function_param_clashing_sub_name_declared_earlier() {
    let program = r#"
            SUB Hello
            END SUB

            FUNCTION Adding(Hello)
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 5, 29);
}

#[test]
fn test_function_param_clashing_sub_name_declared_later() {
    let program = r#"
            FUNCTION Adding(Hello)
            END FUNCTION

            SUB Hello
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
}

#[test]
fn test_function_param_of_different_type_clashing_function_name() {
    let program = r#"
            FUNCTION Adding(Adding$)
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
}

#[test]
fn test_function_param_clashing_function_name_extended_same_type() {
    let program = r#"
            FUNCTION Adding(Adding AS SINGLE)
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 2, 29);
}

#[test]
fn test_function_param_duplicate() {
    let program = r#"
            FUNCTION Adding(Adding, Adding)
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 2, 37);
}

#[test]
fn test_no_args_function_call_cannot_assign_to_variable() {
    let program = r#"
            DECLARE FUNCTION GetAction$

            GetAction% = 42

            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
}

#[test]
fn test_function_call_without_implementation() {
    let program = "
            DECLARE FUNCTION Add(A, B)
            X = Add(1, 2)
            ";
    assert_linter_err!(program, QError::SubprogramNotDefined, 2, 13);
}

#[test]
fn test_cannot_override_built_in_function_with_declaration() {
    let program = r#"
            DECLARE FUNCTION Environ$
            PRINT "Hello"
            FUNCTION Environ$
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
}

#[test]
fn test_cannot_override_built_in_function_without_declaration() {
    let program = r#"
            PRINT "Hello"
            FUNCTION Environ$
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 13);
}

#[test]
fn test_cannot_call_built_in_function_with_wrong_type() {
    let program = r#"
            PRINT "Hello", Environ%("oops")
            "#;
    assert_linter_err!(program, QError::TypeMismatch, 2, 28);
}

#[test]
fn test_function_call_missing_with_string_arguments_gives_type_mismatch() {
    let program = "
            X = Add(\"1\", \"2\")
            ";
    assert_linter_err!(program, QError::ArgumentTypeMismatch, 2, 21);
}

#[test]
fn test_function_dotted_name_clashes_variable_of_user_defined_type() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            DIM A AS Card

            FUNCTION A.B
            END FUNCTION
            ";
    assert_linter_err!(input, QError::DotClash, 8, 22);
}

#[test]
fn test_function_dotted_name_clashes_variable_of_user_defined_type_in_other_function() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            FUNCTION A.B
            END FUNCTION

            FUNCTION C.D
                DIM A AS Card
            END FUNCTION
            ";
    assert_linter_err!(input, QError::DotClash, 6, 22);
}

#[test]
fn test_function_dotted_name_clashes_variable_of_user_defined_type_in_other_function_following() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            FUNCTION A.B
                DIM C AS Card
            END FUNCTION

            FUNCTION C.D
            END FUNCTION
            ";
    assert_linter_err!(input, QError::DotClash, 10, 22);
}

#[test]
fn test_dotted_function_param_clashes_variable_of_user_defined_type() {
    let input = r#"
            TYPE Card
                Value AS INTEGER
            END TYPE

            FUNCTION Hi(A.B)
            END FUNCTION

            DIM A AS Card
            "#;
    assert_linter_err!(input, QError::DotClash, 6, 25);
}

#[test]
fn test_user_defined_function_param_clashes_dotted_variable() {
    let input = r#"
            TYPE Card
                Value AS INTEGER
            END TYPE

            FUNCTION Hi(A AS Card)
            END FUNCTION

            DIM A.B
            "#;
    assert_linter_err!(input, QError::DotClash, 9, 17);
}

#[test]
fn exit_function_not_allowed_in_global_module() {
    let input = "
    EXIT FUNCTION
    ";
    assert_linter_err!(input, QError::syntax_error("Illegal outside of subprogram"));
}

#[test]
fn exit_sub_not_allowed_in_function() {
    let input = "
    FUNCTION Hello
    EXIT SUB
    END FUNCTION
    ";
    assert_linter_err!(input, QError::syntax_error("Illegal inside function"));
}
