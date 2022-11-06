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
fn test_dim_string_max_fixed_length() {
    let program = r#"
    DIM X AS STRING * 32767
    PRINT LEN(X)
    "#;
    assert_prints!(program, "32767");
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

#[test]
fn test_multi_dim() {
    let program = r#"
    DIM A AS STRING, B AS STRING
    A = "hello"
    B = "bye"
    PRINT A, B
    "#;
    assert_prints!(program, "hello         bye");
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

mod redim {
    use super::*;

    /// Test `REDIM` with all possible types
    mod single_definition {
        use super::*;

        #[test]
        fn bare() {
            let input = r#"
            REDIM A(1 TO 5)
            FOR I = 1 TO 5
                A(I) = I * 2
                PRINT A(I)
            NEXT
            "#;
            assert_prints!(input, "2", "4", "6", "8", "10");
        }

        #[test]
        fn compact_single() {
            let input = r#"
            REDIM A!(1 TO 5)
            A!(1) = 3.14
            PRINT A!(1)
            "#;
            assert_prints!(input, "3.14");
        }

        #[test]
        fn compact_double() {
            let input = r#"
            REDIM A#(1 TO 5)
            A#(1) = 3.14
            PRINT A#(1)
            "#;
            assert_prints!(input, "3.140000104904175");
        }

        #[test]
        fn compact_string() {
            let input = r#"
            REDIM A$(1 TO 5)
            A$(1) = "hello"
            PRINT A$(1)
            "#;
            assert_prints!(input, "hello");
        }

        #[test]
        fn compact_integer() {
            let input = r#"
            REDIM A%(1 TO 5)
            A%(1) = 42
            PRINT A%(1)
            "#;
            assert_prints!(input, "42");
        }

        #[test]
        fn compact_long() {
            let input = r#"
            REDIM A&(1 TO 5)
            A&(1) = 65536
            PRINT A&(1)
            "#;
            assert_prints!(input, "65536");
        }

        #[test]
        fn extended_single() {
            let input = r#"
            REDIM A(1 TO 5) AS SINGLE
            A(1) = 3.14
            PRINT A(1)
            "#;
            assert_prints!(input, "3.14");
        }

        #[test]
        fn extended_double() {
            let input = r#"
            REDIM A(1 TO 5) AS DOUBLE
            A(1) = 3.14
            PRINT A(1)
            "#;
            assert_prints!(input, "3.140000104904175");
        }

        #[test]
        fn extended_string() {
            let input = r#"
            REDIM A(1 TO 5) AS STRING
            A(1) = "hello"
            PRINT A(1)
            "#;
            assert_prints!(input, "hello");
        }

        #[test]
        fn extended_integer() {
            let input = r#"
            REDIM A(1 TO 5)
            A(1) = 42
            PRINT A(1)
            "#;
            assert_prints!(input, "42");
        }

        #[test]
        fn extended_long() {
            let input = r#"
            REDIM A&(1 TO 5)
            A&(1) = 65536
            PRINT A&(1)
            "#;
            assert_prints!(input, "65536");
        }

        #[test]
        fn fixed_length_string() {
            let input = r#"
            REDIM A(1 TO 5) AS STRING * 2
            A(1) = "hello"
            PRINT A(1)
            "#;
            assert_prints!(input, "he");
        }

        #[test]
        fn user_defined_type() {
            let input = r#"
            TYPE Card
                Value AS INTEGER
                Suit AS STRING * 4
            END TYPE
            REDIM A(1 TO 5) AS Card
            A(1).Value = 42
            A(1).Suit = "diamonds"
            PRINT A(1).Value ; A(1).Suit
            "#;
            assert_prints!(input, "42 diam");
        }
    }

    #[test]
    fn test_redim_change_single_dimension_array_of_bare_type() {
        let input = r#"
        REDIM A(1 TO 2)
        FOR I = 1 TO 2
            A(I) = 10 + I
        NEXT

        REDIM A(1 TO 3)
        FOR I = 1 TO 3
            PRINT A(I)
        NEXT
        "#;
        assert_prints!(input, "0", "0", "0");
    }

    #[test]
    fn test_redim_change_single_dimension_array_of_user_defined_type() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
        END TYPE
        REDIM A(1 TO 2) AS Card
        FOR I = 1 TO 2
            A(I).Value = 10 + I
        NEXT

        REDIM A(1 TO 3) AS Card
        FOR I = 1 TO 3
            PRINT A(I).Value
        NEXT
        "#;
        assert_prints!(input, "0", "0", "0");
    }

    #[test]
    fn test_redim_without_specifying_type_keeps_old_type() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
            Suit AS STRING * 5
        END TYPE

        REDIM A(1 TO 5) AS Card
        REDIM A(1 TO 2)
        A(1).Value = 42
        PRINT A(1).Value
        "#;
        assert_prints!(input, "42");
    }
}
