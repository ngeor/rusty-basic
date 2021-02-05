use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;

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

mod dim_shared {
    use super::*;

    #[test]
    fn test_dim_shared_bare() {
        let program = r#"
        DIM SHARED A
        A = 42
        PRINT A
        SubThatUsesSharedVariable
        PRINT A

        SUB SubThatUsesSharedVariable
            A = 3
        END SUB
        "#;
        assert_prints!(program, "42", "3");
    }

    #[test]
    fn test_dim_shared_compact_string() {
        let program = r#"
        DIM SHARED A$
        A$ = "hi"
        PRINT A$
        SubThatUsesSharedVariable
        PRINT A$

        SUB SubThatUsesSharedVariable
            A$ = "hello"
        END SUB
        "#;
        assert_prints!(program, "hi", "hello");
    }

    #[test]
    fn test_dim_shared_extended_string() {
        let program = r#"
        DIM SHARED A AS STRING
        A = "hi"
        PRINT A
        SubThatUsesSharedVariable
        PRINT A

        SUB SubThatUsesSharedVariable
            A = "hello"
        END SUB
        "#;
        assert_prints!(program, "hi", "hello");
    }

    #[test]
    fn test_dim_shared_user_defined_type() {
        let program = r#"
        TYPE User
            Username AS STRING * 10
        END TYPE
        DIM SHARED A AS User
        A.Username = "hi"
        PRINT A.Username
        SubThatUsesSharedVariable
        PRINT A.Username

        SUB SubThatUsesSharedVariable
            A.Username = "hello"
        END SUB
        "#;
        assert_prints!(program, "hi", "hello");
    }

    #[test]
    fn test_dim_shared_array_bare() {
        let program = r#"
        DIM SHARED A(5)
        A(1) = 42
        PRINT A(1)
        SubThatUsesSharedVariable
        PRINT A(1)

        SUB SubThatUsesSharedVariable
            A(1) = 12
        END SUB
        "#;
        assert_prints!(program, "42", "12");
    }

    #[test]
    fn test_dim_shared_array_compact_string() {
        let program = r#"
        DIM SHARED A$(5)
        A$(1) = "hi"
        PRINT A$(1)
        SubThatUsesSharedVariable
        PRINT A$(1)

        SUB SubThatUsesSharedVariable
            A$(1) = "hello"
        END SUB
        "#;
        assert_prints!(program, "hi", "hello");
    }

    #[test]
    fn test_dim_shared_array_extended_string() {
        let program = r#"
        DIM SHARED A(5) AS STRING
        A(1) = "hi"
        PRINT A(1)
        SubThatUsesSharedVariable
        PRINT A(1)

        SUB SubThatUsesSharedVariable
            A(1) = "hello"
        END SUB
        "#;
        assert_prints!(program, "hi", "hello");
    }

    #[test]
    fn test_dim_shared_array_user_defined_type() {
        let program = r#"
        TYPE User
            Username AS STRING * 10
        END TYPE
        DIM SHARED A(5) AS User
        A(1).Username = "hi"
        PRINT A(1).Username
        SubThatUsesSharedVariable
        PRINT A(1).Username

        SUB SubThatUsesSharedVariable
            A(1).Username = "hello"
        END SUB
        "#;
        assert_prints!(program, "hi", "hello");
    }
}
