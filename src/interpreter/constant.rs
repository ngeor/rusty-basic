#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_linter_err;
    use crate::linter::LinterError;

    mod unqualified_integer_declaration {
        use super::*;

        #[test]
        fn unqualified_usage() {
            let program = "
            CONST X = 42
            PRINT X
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["42"]);
        }

        #[test]
        fn qualified_usage() {
            let program = "
            CONST X = 42
            PRINT X%
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["42"]);
        }

        #[test]
        fn qualified_usage_wrong_type() {
            let program = "
            CONST X = 42
            PRINT X!
            ";
            assert_linter_err!(program, LinterError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn variable_already_exists() {
            let program = "
            X = 42
            CONST X = 32
            PRINT X
            ";
            assert_linter_err!(program, LinterError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn variable_already_exists_as_sub_call_param() {
            let program = "
            INPUT X%
            CONST X = 1
            ";
            assert_linter_err!(program, LinterError::DuplicateDefinition, 3, 19);
        }

        #[test]
        fn const_already_exists() {
            let program = "
            CONST X = 32
            CONST X = 33
            PRINT X
            ";
            assert_linter_err!(program, LinterError::DuplicateDefinition, 3, 19);
        }
    }

    mod unqualified_single_declaration {
        use super::*;

        #[test]
        fn unqualified_usage() {
            let program = "
            CONST X = 3.14
            PRINT X
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3.14"]);
        }

        #[test]
        fn qualified_usage() {
            let program = r#"
            CONST X = 3.14
            PRINT X!
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3.14"]);
        }

        #[test]
        fn assign_is_duplicate_definition() {
            let program = "
            CONST X = 3.14
            X = 6.28
            ";
            assert_linter_err!(program, LinterError::DuplicateDefinition, 3, 13);
        }
    }

    mod unqualified_double_declaration {
        use super::*;

        #[test]
        fn unqualified_usage() {
            let program = "
            CONST X = 3.14#
            PRINT X
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3.14"]);
        }
    }

    mod unqualified_string_declaration {
        use super::*;

        #[test]
        fn unqualified_usage() {
            let program = r#"
            CONST X = "hello"
            PRINT X
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["hello"]);
        }
    }

    mod qualified_single_declaration {
        use super::*;

        #[test]
        fn qualified_usage_casting_from_integer() {
            let program = "
            CONST X! = 42
            PRINT X!
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["42"]);
        }

        #[test]
        fn qualified_usage_from_single_literal() {
            let program = "
            CONST X! = 3.14
            PRINT X!
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3.14"]);
        }

        #[test]
        fn qualified_usage_from_string_literal() {
            let program = r#"
            CONST X! = "hello"
            PRINT X!
            "#;
            assert_linter_err!(program, LinterError::TypeMismatch, 2, 24);
        }
    }

    mod qualified_integer_declaration {
        use super::*;

        #[test]
        fn unqualified_usage_type_mismatch() {
            let program = r#"
            CONST X% = "hello"
            PRINT X
            "#;
            assert_linter_err!(program, LinterError::TypeMismatch, 2, 24);
        }
    }

    mod expressions {
        use super::*;

        #[test]
        fn binary_plus() {
            let program = r#"
            CONST X = 1
            CONST Y = X + 2
            PRINT Y
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3"]);
        }

        #[test]
        fn binary_minus() {
            let program = r#"
            CONST X = 3
            CONST Y = X - 2
            PRINT Y
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["1"]);
        }

        #[test]
        fn unary_minus() {
            let program = r#"
            CONST X = 3
            CONST Y = -X
            PRINT Y
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["-3"]);
        }

        #[test]
        fn unary_not() {
            let program = r#"
            CONST TRUE = -1
            CONST FALSE = NOT TRUE
            PRINT FALSE
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["0"]);
        }

        #[test]
        fn function_call_not_allowed() {
            let program = r#"
            CONST X = Add(1, 2)
            PRINT X
            "#;
            assert_linter_err!(program, LinterError::InvalidConstant, 2, 23);
        }

        #[test]
        fn variable_not_allowed() {
            let program = r#"
            X = 42
            CONST A = X + 1
            PRINT A
            "#;
            assert_linter_err!(program, LinterError::InvalidConstant, 3, 23);
        }
    }

    mod sub_usage {
        use super::*;

        #[test]
        fn simple_usage() {
            let program = r#"
            CONST X = 42
            DECLARE SUB Hello

            Hello

            SUB Hello
                PRINT X
            END SUB
            "#;

            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["42"]);
        }

        #[test]
        fn parameter_hides_const() {
            let program = r#"
            CONST X = 42
            DECLARE SUB Hello(X)

            Hello 5

            SUB Hello(X)
                PRINT X
            END SUB
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["5"]);
        }

        #[test]
        fn redefine() {
            let program = r#"
            CONST X = 42
            DECLARE SUB Hello

            Hello
            PRINT X

            SUB Hello
                PRINT X
                CONST X = 100
                PRINT X
            END SUB
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["42", "100", "42"]);
        }

        #[test]
        fn nested_sub() {
            let program = "
            CONST X = 42
            Sub1

            SUB Sub1
                CONST X = 3
                PRINT X
                Sub2
            END SUB

            SUB Sub2
                PRINT X
            END SUB
            ";
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.output, vec!["3", "42"]);
        }
    }
}
