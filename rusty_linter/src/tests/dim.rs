use crate::assert_linter_err;
use crate::test_utils::linter_ok;
use crate::LintError;
use rusty_common::*;
use rusty_parser::*;

#[test]
fn test_dim_duplicate_definition_same_builtin_type() {
    let program = r#"
            DIM A AS STRING
            DIM A AS STRING
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_duplicate_definition_different_builtin_type() {
    let program = r#"
            DIM A AS STRING
            DIM A AS INTEGER
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_after_const_duplicate_definition() {
    let program = r#"
            CONST A = "hello"
            DIM A AS STRING
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_after_variable_assignment_duplicate_definition() {
    let program = r#"
            A = 42
            DIM A AS INTEGER
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_string_duplicate_definition() {
    let program = r#"
            DIM A$
            DIM A$
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_bare_duplicate_definition() {
    let program = r#"
            DIM A
            DIM A
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_single_bare_duplicate_definition() {
    // single is the default type
    let program = r#"
            DIM A!
            DIM A
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_bare_single_duplicate_definition() {
    // single is the default type
    let program = r#"
            DIM A
            DIM A!
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_bare_integer_duplicate_definition() {
    let program = r#"
            DEFINT A-Z
            DIM A
            DIM A%
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 4, 17);
}

#[test]
fn test_dim_extended_inside_sub_name_clashing_sub_name() {
    let program = r#"
            SUB Hello
            Dim Hello AS STRING
            END SUB
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_bare_inside_sub_name_clashing_other_sub_name() {
    let program = r#"
            SUB Oops
            END SUB

            SUB Hello
            Dim Oops
            END SUB
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 6, 17);
}

#[test]
fn test_dim_extended_inside_sub_name_clashing_param_name() {
    let program = r#"
            SUB Hello(Oops)
            Dim Oops AS STRING
            END SUB
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_extended_inside_function_name_clashing_function_name() {
    let program = r#"
            FUNCTION Hello
            Dim Hello AS STRING
            END FUNCTION
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_extended_inside_function_name_clashing_other_function_name() {
    let program = r#"
            FUNCTION Hello
            Dim Bar AS STRING
            END FUNCTION
            FUNCTION Bar
            END Function
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_bare() {
    assert_eq!(
        linter_ok("DIM A"),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimVar::new_compact_local("A", TypeQualifier::BangSingle).into_list_rc(1, 5)
        ))
        .at_rc(1, 1)]
    );
}

#[test]
fn test_dim_qualified() {
    assert_eq!(
        linter_ok("DIM A$"),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimVar::new_compact_local("A", TypeQualifier::DollarString).into_list_rc(1, 5)
        ))
        .at_rc(1, 1)]
    );
}

#[test]
fn test_dim_extended_built_in() {
    assert_eq!(
        linter_ok("DIM A AS LONG"),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimNameBuilder::new()
                .bare_name("A")
                .dim_type(DimType::BuiltIn(
                    TypeQualifier::AmpersandLong,
                    BuiltInStyle::Extended
                ))
                .build_list_rc(1, 5)
        ))
        .at_rc(1, 1)]
    );
}

#[test]
fn test_dim_extended_fixed_length_string() {
    assert_eq!(
        linter_ok("DIM A AS STRING * 5"),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimNameBuilder::new()
                .bare_name("A")
                .dim_type(DimType::FixedLengthString(
                    Expression::IntegerLiteral(5).at_rc(1, 19),
                    5
                ))
                .build_list_rc(1, 5)
        ))
        .at_rc(1, 1)]
    );
}

#[test]
fn test_dim_extended_user_defined() {
    let input = r#"
    TYPE Card
        Value AS INTEGER
    END TYPE
    DIM A AS Card
    "#;
    assert_eq!(
        linter_ok(input),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimNameBuilder::new()
                .bare_name("A")
                .dim_type(DimType::UserDefined(BareName::from("Card").at_rc(5, 14)))
                .build_list_rc(5, 9)
        ))
        .at_rc(5, 5)]
    );
}

#[test]
fn test_dim_array_bare() {
    assert_eq!(
        linter_ok("DIM A(2)"),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimNameBuilder::new()
                .bare_name("A")
                .dim_type(DimType::Array(
                    vec![ArrayDimension {
                        lbound: None,
                        ubound: Expression::IntegerLiteral(2).at_rc(1, 7)
                    }],
                    Box::new(DimType::BuiltIn(
                        TypeQualifier::BangSingle,
                        BuiltInStyle::Compact
                    ))
                ))
                .build_list_rc(1, 5)
        ))
        .at_rc(1, 1)]
    );
}

#[test]
fn test_dim_array_qualified() {
    assert_eq!(
        linter_ok("DIM A$(2)"),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimNameBuilder::new()
                .bare_name("A")
                .dim_type(DimType::Array(
                    vec![ArrayDimension {
                        lbound: None,
                        ubound: Expression::IntegerLiteral(2).at_rc(1, 8)
                    }],
                    Box::new(DimType::BuiltIn(
                        TypeQualifier::DollarString,
                        BuiltInStyle::Compact
                    ))
                ))
                .build_list_rc(1, 5)
        ))
        .at_rc(1, 1)]
    );
}

#[test]
fn test_dim_array_extended_built_in() {
    assert_eq!(
        linter_ok("DIM A(2) AS INTEGER"),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimNameBuilder::new()
                .bare_name("A")
                .dim_type(DimType::Array(
                    vec![ArrayDimension {
                        lbound: None,
                        ubound: Expression::IntegerLiteral(2).at_rc(1, 7)
                    }],
                    Box::new(DimType::BuiltIn(
                        TypeQualifier::PercentInteger,
                        BuiltInStyle::Extended
                    ))
                ))
                .build_list_rc(1, 5)
        ))
        .at_rc(1, 1)]
    );
}

#[test]
fn test_dim_array_extended_fixed_length_string() {
    assert_eq!(
        linter_ok("DIM A(2) AS STRING * 3"),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimNameBuilder::new()
                .bare_name("A")
                .dim_type(DimType::Array(
                    vec![ArrayDimension {
                        lbound: None,
                        ubound: Expression::IntegerLiteral(2).at_rc(1, 7)
                    }],
                    Box::new(DimType::FixedLengthString(
                        Expression::IntegerLiteral(3).at_rc(1, 22),
                        3
                    ))
                ))
                .build_list_rc(1, 5)
        ))
        .at_rc(1, 1)]
    );
}

#[test]
fn test_dim_array_extended_user_defined() {
    let input = r#"
    TYPE Card
        Value AS INTEGER
    END TYPE
    DIM A(2) AS Card
    "#;
    assert_eq!(
        linter_ok(input),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimNameBuilder::new()
                .bare_name("A")
                .dim_type(DimType::Array(
                    vec![ArrayDimension {
                        lbound: None,
                        ubound: Expression::IntegerLiteral(2).at_rc(5, 11)
                    }],
                    Box::new(DimType::UserDefined(BareName::from("Card").at_rc(5, 17)))
                ))
                .build_list_rc(5, 9)
        ))
        .at_rc(5, 5)]
    );
}

mod dot_clash {
    use super::*;

    #[test]
    fn test_clash_with_user_defined_type() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
        END TYPE
        DIM A AS Card
        DIM A.B
        "#;
        assert_linter_err!(input, LintError::DotClash, 6, 13);
    }

    #[test]
    fn test_no_clash_with_built_in_extended() {
        let input = r#"
        DIM A AS INTEGER
        DIM A.B
        "#;
        linter_ok(input);
    }

    #[test]
    fn test_no_clash_with_string_length() {
        let input = r#"
        DIM A AS STRING * 5
        DIM A.B
        "#;
        linter_ok(input);
    }
}

mod dim_shared {
    use super::*;

    #[test]
    fn test_dim_shared_in_function_not_allowed() {
        let program = r#"
        FUNCTION Test
            DIM SHARED A
        END FUNCTION
        "#;
        assert_linter_err!(program, LintError::IllegalInSubFunction, 3, 24);
    }

    #[test]
    fn test_dim_shared_in_sub_not_allowed() {
        let program = r#"
        SUB Test
            DIM SHARED A
        END SUB
        "#;
        assert_linter_err!(program, LintError::IllegalInSubFunction, 3, 24);
    }

    #[test]
    fn test_dim_in_function_clash_with_shared_dim() {
        let program = r#"
        DIM SHARED A AS STRING
        FUNCTION Test
            DIM A AS STRING
        END FUNCTION
        "#;
        assert_linter_err!(program, LintError::DuplicateDefinition, 4, 17);
    }

    #[test]
    fn test_dim_in_sub_clash_with_shared_dim() {
        let program = r#"
        DIM SHARED A AS STRING
        SUB Test
            DIM A AS STRING
        END SUB
        "#;
        assert_linter_err!(program, LintError::DuplicateDefinition, 4, 17);
    }

    #[test]
    fn test_const_in_function_clash_with_shared_dim() {
        let program = r#"
        DIM SHARED A AS STRING
        FUNCTION Test
            CONST A = "hello"
        END FUNCTION
        "#;
        assert_linter_err!(program, LintError::DuplicateDefinition, 4, 19);
    }

    #[test]
    fn test_const_in_sub_clash_with_shared_dim() {
        let program = r#"
        DIM SHARED A AS STRING
        SUB Test
            CONST A = "hello"
        END SUB
        "#;
        assert_linter_err!(program, LintError::DuplicateDefinition, 4, 19);
    }
}

mod redim {
    use super::*;

    mod const_exists {
        use super::*;

        #[test]
        fn bare() {
            let input = r#"
            CONST A = 42
            REDIM A(1 TO 5)
            "#;
            assert_linter_err!(input, LintError::DuplicateDefinition);
        }

        #[test]
        fn compacts() {
            for ch in "!#$%&".chars() {
                let input = format!(
                    r#"
                    CONST A = 42
                    REDIM A{}(1 TO 5)
                    "#,
                    ch
                );
                assert_linter_err!(&input, LintError::DuplicateDefinition);
            }
        }

        #[test]
        fn extended() {
            for s in &[
                "SINGLE",
                "DOUBLE",
                "STRING",
                "INTEGER",
                "LONG",
                "STRING * 5",
                "Card",
            ] {
                let input = format!(
                    r#"
                    TYPE Card
                        Value AS INTEGER
                    END TYPE

                    CONST A = 42
                    REDIM A(1 TO 5) AS {}
                    "#,
                    s
                );
                assert_linter_err!(&input, LintError::DuplicateDefinition, s);
            }
        }
    }

    mod non_array_variable_exists {
        use super::*;

        #[test]
        fn bare() {
            for s in &[
                "A",
                "A AS SINGLE",
                "A AS DOUBLE",
                "A AS STRING",
                "A AS INTEGER",
                "A AS LONG",
                "A AS STRING * 5",
                "A AS Card",
            ] {
                let input = format!(
                    r#"
                TYPE Card
                    Value AS INTEGER
                END TYPE

                DIM {}
                REDIM A(1 TO 5)
                "#,
                    s
                );
                assert_linter_err!(&input, LintError::DuplicateDefinition, s);
            }
        }

        #[test]
        fn redim_as_compact_when_compact_of_same_type_exists() {
            for ch in "!#$%&".chars() {
                let input = format!(
                    r#"
                DIM A{}
                REDIM A{}(1 TO 5)
                "#,
                    ch, ch
                );
                assert_linter_err!(&input, LintError::DuplicateDefinition, ch);
            }
        }

        #[test]
        fn redim_as_compact_when_extended_exists() {
            for ch in "!#$%&".chars() {
                for s in &[
                    "SINGLE",
                    "DOUBLE",
                    "STRING",
                    "INTEGER",
                    "LONG",
                    "STRING * 5",
                    "Card",
                ] {
                    let input = format!(
                        r#"
                    TYPE Card
                        Value AS INTEGER
                    END TYPE
                    DIM A AS {}
                    REDIM A{}(1 TO 5)
                    "#,
                        s, ch
                    );
                    assert_linter_err!(&input, LintError::DuplicateDefinition);
                }
            }
        }

        #[test]
        fn redim_as_extended_when_compact_exists() {
            for ch in "!#$%&".chars() {
                for s in &[
                    "SINGLE",
                    "DOUBLE",
                    "STRING",
                    "INTEGER",
                    "LONG",
                    "STRING * 5",
                    "Card",
                ] {
                    let input = format!(
                        r#"
                    TYPE Card
                        Value AS INTEGER
                    END TYPE
                    DIM A{}
                    REDIM A(1 TO 5) AS {}
                    "#,
                        ch, s
                    );
                    assert_linter_err!(&input, LintError::DuplicateDefinition);
                }
            }
        }

        #[test]
        fn redim_as_extended_when_extended_exists() {
            for s in &[
                "SINGLE",
                "DOUBLE",
                "STRING",
                "INTEGER",
                "LONG",
                "STRING * 5",
                "Card",
            ] {
                for s2 in &[
                    "SINGLE",
                    "DOUBLE",
                    "STRING",
                    "INTEGER",
                    "LONG",
                    "STRING * 5",
                    "Card",
                ] {
                    let input = format!(
                        r#"
                    TYPE Card
                        Value AS INTEGER
                    END TYPE
                    DIM A AS {}
                    REDIM A(1 TO 5) AS {}
                    "#,
                        s, s2
                    );
                    assert_linter_err!(
                        &input,
                        LintError::DuplicateDefinition,
                        format!("{} -> {}", s, s2)
                    );
                }
            }
        }
    }

    mod already_dimensioned {
        use super::*;

        #[test]
        fn compacts() {
            for ch in "!#$%&".chars() {
                let input = format!(
                    r#"
                DIM A{}(1 TO 5)
                REDIM A{}(1 TO 5)
                "#,
                    ch, ch
                );
                assert_linter_err!(&input, LintError::ArrayAlreadyDimensioned);
            }
        }

        #[test]
        fn extended() {
            for s in &[
                "SINGLE",
                "DOUBLE",
                "STRING",
                "INTEGER",
                "LONG",
                "STRING * 5",
                "Card",
            ] {
                let input = format!(
                    r#"
                TYPE Card
                    Value AS INTEGER
                END TYPE
                DIM A(1 TO 5) AS {}
                REDIM A(1 TO 5) AS {}
                "#,
                    s, s
                );
                assert_linter_err!(&input, LintError::ArrayAlreadyDimensioned, s);
            }
        }
    }

    mod wrong_number_of_dimensions {
        use super::*;

        #[test]
        fn compacts() {
            for ch in "!#$%&".chars() {
                let input = format!(
                    r#"
                REDIM A{}(1 TO 5)
                REDIM A{}(1 TO 5, 1 TO 10)
                "#,
                    ch, ch
                );
                assert_linter_err!(&input, LintError::WrongNumberOfDimensions);
            }
        }

        #[test]
        fn extended() {
            for s in &[
                "SINGLE",
                "DOUBLE",
                "STRING",
                "INTEGER",
                "LONG",
                "STRING * 5",
                "Card",
            ] {
                let input = format!(
                    r#"
                TYPE Card
                    Value AS INTEGER
                END TYPE
                REDIM A(1 TO 5) AS {}
                REDIM A(1 TO 5, 1 TO 10) AS {}
                "#,
                    s, s
                );
                assert_linter_err!(&input, LintError::WrongNumberOfDimensions, s);
            }
        }
    }

    mod cannot_change_type {
        use super::*;

        #[test]
        fn cannot_change_type_extended_to_extended() {
            let types = &[
                "SINGLE",
                "DOUBLE",
                "STRING",
                "INTEGER",
                "LONG",
                "STRING * 5",
                "STRING * 10",
                "Card",
                "PostCode",
            ];
            for a in types {
                for b in types {
                    if a != b {
                        let input = format!(
                            r#"
                            TYPE Card
                                Value AS INTEGER
                            END TYPE

                            TYPE PostCode
                                Suffix AS STRING * 4
                            END TYPE

                            REDIM A(1 TO 5) AS {}
                            REDIM A(1 TO 5) AS {}
                            "#,
                            a, b
                        );
                        assert_linter_err!(
                            &input,
                            LintError::DuplicateDefinition,
                            format!("REDIM {} to {}", a, b).as_str()
                        );
                    }
                }
            }
        }
    }
}
