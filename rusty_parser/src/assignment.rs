#[cfg(test)]
mod tests {
    use crate::assert_global_assignment;
    use crate::assert_parser_err;
    use crate::test_utils::*;
    use crate::*;
    use rusty_common::*;

    mod unqualified {
        use super::*;

        mod no_dots {
            use super::*;

            #[test]
            fn test_assign_unqualified_variable_no_dots() {
                let input = "A = 42";
                assert_global_assignment!(input, Expression::var_unresolved("A"));
            }

            #[test]
            fn test_whitespace_around_equals_is_optional() {
                let var_name = "A";
                let value = 42;
                assert_global_assignment!(&format!("{} = {}", var_name, value), var_name, value);
                assert_global_assignment!(&format!("{}={}", var_name, value), var_name, value);
                assert_global_assignment!(&format!("{}= {}", var_name, value), var_name, value);
                assert_global_assignment!(&format!("{} ={}", var_name, value), var_name, value);
            }

            #[test]
            fn test_assign_unqualified_variable_no_dots_array() {
                let input = "A(1) = 42";
                assert_global_assignment!(
                    input,
                    Expression::FunctionCall("A".into(), vec![1.as_lit_expr(1, 3)])
                );
            }
        }

        mod dots {
            use super::*;

            #[test]
            fn test_potential_property() {
                let input = "A.B = 42";
                assert_global_assignment!(
                    input,
                    Expression::Property(
                        Box::new(Expression::var_unresolved("A")),
                        "B".into(),
                        ExpressionType::Unresolved
                    )
                );
            }

            #[test]
            fn test_not_property_due_to_consecutive_dots() {
                let input = "A..B = 42";
                assert_global_assignment!(input, Expression::var_unresolved("A..B"));
            }

            #[test]
            fn test_assign_array_property() {
                let input = "A(1).Value = 42";
                assert_global_assignment!(
                    input,
                    Expression::Property(
                        Box::new(Expression::FunctionCall(
                            "A".into(),
                            vec![1.as_lit_expr(1, 3)]
                        )),
                        "Value".into(),
                        ExpressionType::Unresolved
                    )
                );
            }

            #[test]
            fn test_max_length() {
                assert_global_assignment!(
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZ.ABCDEFGHIJKLM = 42",
                    Expression::Property(
                        Box::new(Expression::var_unresolved("ABCDEFGHIJKLMNOPQRSTUVWXYZ")),
                        "ABCDEFGHIJKLM".into(),
                        ExpressionType::Unresolved
                    )
                );
            }
        }
    }

    mod qualified {
        use super::*;

        mod no_dots {
            use super::*;

            #[test]
            fn test_assign_qualified_variable_no_dots() {
                let input = "A% = 42";
                assert_global_assignment!(input, Expression::var_unresolved("A%"));
            }

            #[test]
            fn test_assign_qualified_variable_no_dots_array() {
                let input = "A%(1) = 42";
                assert_global_assignment!(
                    input,
                    Expression::FunctionCall("A%".into(), vec![1.as_lit_expr(1, 4)])
                );
            }
        }

        mod dots {
            use super::*;

            #[test]
            fn test_assign_array_property() {
                let input = "A(1).Value% = 42";
                assert_global_assignment!(
                    input,
                    Expression::Property(
                        Box::new(Expression::FunctionCall(
                            "A".into(),
                            vec![1.as_lit_expr(1, 3)]
                        )),
                        "Value%".into(),
                        ExpressionType::Unresolved
                    )
                );
            }

            #[test]
            fn test_max_length() {
                assert_global_assignment!(
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZ.ABCDEFGHIJKLM% = 42",
                    Expression::Property(
                        Box::new(Expression::var_unresolved("ABCDEFGHIJKLMNOPQRSTUVWXYZ")),
                        "ABCDEFGHIJKLM%".into(),
                        ExpressionType::Unresolved
                    )
                );
            }

            #[test]
            fn test_max_length_variable_with_trailing_dot() {
                assert_global_assignment!(
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLM.% = 42",
                    Expression::Variable(
                        "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLM.%".into(),
                        VariableInfo::unresolved()
                    )
                );
            }
        }
    }

    #[test]
    fn test_numeric_assignment() {
        let names = ["A", "BC", "A%", "A..B", "A.B.", "C.%"];
        let values = [1, -1, 0, 42];
        for name in &names {
            for value in &values {
                assert_global_assignment!(&format!("{} = {}", name, value), *name, *value);
                assert_global_assignment!(&format!("{}={}", name, value), *name, *value);
                assert_global_assignment!(&format!("{} ={}", name, value), *name, *value);
                assert_global_assignment!(&format!("{}= {}", name, value), *name, *value);
            }
        }
    }

    #[test]
    fn test_numeric_assignment_to_keyword_not_allowed() {
        assert_parser_err!(
            "FOR = 42",
            ParseError::syntax_error("Expected: name after FOR")
        );
    }

    #[test]
    fn test_identifier_too_long() {
        assert_parser_err!(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ.ABCDEFGHIJKLMN = 42",
            ParseError::IdentifierTooLong
        );
        assert_parser_err!(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ.ABCDEFGHIJKLMN% = 42",
            ParseError::IdentifierTooLong
        );
    }

    #[test]
    fn test_numeric_assignment_to_keyword_plus_number_allowed() {
        assert_global_assignment!("FOR42 = 42", "FOR42", 42);
    }

    #[test]
    fn test_inline_comment() {
        let input = "ANSWER = 42 ' the answer to life, universe, everything";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::Assignment(
                    Expression::var_unresolved("ANSWER"),
                    42.as_lit_expr(1, 10)
                ))
                .at_rc(1, 1),
                GlobalStatement::Statement(Statement::Comment(
                    " the answer to life, universe, everything".to_string(),
                ))
                .at_rc(1, 13)
            ]
        );
    }

    #[test]
    fn test_array_with_single_dimension() {
        let input = "A(2) = 1";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::func("A", vec![2.as_lit_expr(1, 3)]),
                1.as_lit_expr(1, 8)
            )
        );
    }

    #[test]
    fn test_array_with_two_dimensions() {
        let input = "A(1, 2) = 3";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::func("A", vec![1.as_lit_expr(1, 3), 2.as_lit_expr(1, 6)]),
                3.as_lit_expr(1, 11)
            )
        );
    }

    #[test]
    fn test_array_qualified() {
        let input = "A$(N!) = 1";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::func("A$", vec!["N!".as_var_expr(1, 4)]),
                1.as_lit_expr(1, 10)
            )
        );
    }

    #[test]
    fn test_array_with_user_defined_type_element() {
        let input = "A(1).Height = 2";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::Property(
                    Box::new(Expression::func("A", vec![1.as_lit_expr(1, 3)])),
                    "Height".into(),
                    ExpressionType::Unresolved
                ),
                2.as_lit_expr(1, 15)
            )
        );
    }

    #[test]
    fn test_unqualified_user_defined_type_element() {
        let input = "A.B = 2";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::Property(
                    Box::new(Expression::var_unresolved("A")),
                    "B".into(),
                    ExpressionType::Unresolved
                ),
                2.as_lit_expr(1, 7)
            )
        );
    }

    // edge case where DIM$ = "hello" is allowed but not DIM = 42
    mod keyword_qualified_by_string_is_allowed {
        use super::*;

        #[test]
        fn test_can_assign_to_keyword_qualified_by_string() {
            let input = "DIM$ = \"hello\"";
            let program = parse(input).demand_single_statement();
            assert_eq!(
                program,
                Statement::Assignment(
                    Expression::Variable("DIM$".into(), VariableInfo::unresolved()),
                    "hello".as_lit_expr(1, 8)
                )
            );
        }

        #[test]
        fn test_cannot_assign_to_other_cases_of_keyword() {
            let left_sides = ["DIM", "DIM%", "DIM&", "DIM!", "DIM#"];
            for left_side in &left_sides {
                let input = format!("{} = 42", left_side);
                assert!(matches!(parse_err(&input), ParseError::SyntaxError(_)));
            }
        }
    }
}
