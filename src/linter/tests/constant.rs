use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn function_call_not_allowed() {
    let program = r#"
            CONST X = Add(1, 2)
            "#;
    assert_linter_err!(program, QError::InvalidConstant, 2, 23);
}

#[test]
fn variable_not_allowed() {
    let program = r#"
            X = 42
            CONST A = X + 1
            "#;
    assert_linter_err!(program, QError::InvalidConstant, 3, 23);
}

#[test]
fn variable_already_exists() {
    let program = "
            X = 42
            CONST X = 32
            ";
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
}

#[test]
fn variable_already_exists_as_sub_call_param() {
    let program = "
            INPUT X%
            CONST X = 1
            ";
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
}

#[test]
fn const_already_exists() {
    let program = "
            CONST X = 32
            CONST X = 33
            ";
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
}

#[test]
fn qualified_usage_from_string_literal() {
    let program = r#"
            CONST X! = "hello"
            "#;
    assert_linter_err!(program, QError::TypeMismatch, 2, 24);
}

#[test]
fn const_after_dim_duplicate_definition() {
    let program = r#"
            DIM A AS STRING
            CONST A = "hello"
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
}

#[test]
fn test_global_const_cannot_have_function_name() {
    let program = r#"
            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            CONST GetAction = 42
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 5, 19);
}

#[test]
fn test_local_const_cannot_have_function_name() {
    let program = r#"
            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            FUNCTION Echo(X)
                CONST GetAction = 42
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 6, 23);
}

#[test]
fn test_forward_const_not_allowed() {
    let input = "
            CONST A = B + 1
            CONST B = 42";
    assert_linter_err!(input, QError::InvalidConstant, 2, 23);
}
