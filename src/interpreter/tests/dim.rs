use crate::assert_prints;
use crate::interpreter::interpreter::InterpreterTrait;

#[test]
fn test_dim_string() {
    let program = r#"
    DIM A AS STRING
    A = "hello"
    PRINT A
    "#;
    assert_prints!(program, "hello");
}

#[test]
fn test_dim_implicit_multiple_types_one_dim_one_assignment() {
    let program = r#"
    DIM A$
    A% = 42
    A$ = "hello"
    PRINT A$
    PRINT A%
    "#;
    assert_prints!(program, "hello", "42");
}

#[test]
fn test_dim_implicit_multiple_types_two_dims() {
    let program = r#"
    DIM A$
    DIM A%
    A% = 42
    A$ = "hello"
    PRINT A$
    PRINT A%
    "#;
    assert_prints!(program, "hello", "42");
}

#[test]
fn test_dim_string_fixed_length() {
    let program = r#"
    DIM X AS STRING * 5
    X = "123456"
    PRINT X
    "#;
    assert_prints!(program, "12345");
}

#[test]
fn test_dim_string_fixed_length_length_declared_as_const() {
    let program = r#"
    CONST A = 5
    DIM X AS STRING * A
    X = "123456"
    PRINT X
    "#;
    assert_prints!(program, "12345");
}
