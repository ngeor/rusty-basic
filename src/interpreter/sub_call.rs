#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::common::*;
    use crate::interpreter::test_utils::*;
    use crate::interpreter::{InterpreterError, Stdlib};
    use crate::linter::LinterError;

    mod input {
        mod unqualified_var {
            use crate::interpreter::test_utils::*;

            #[test]
            fn test_input_empty() {
                assert_input("", "N", "N!", 0.0_f32);
            }

            #[test]
            fn test_input_zero() {
                assert_input("0", "N", "N!", 0.0_f32);
            }

            #[test]
            fn test_input_single() {
                assert_input("1.1", "N", "N!", 1.1_f32);
            }

            #[test]
            fn test_input_negative() {
                assert_input("-1.2345", "N", "N!", -1.2345_f32);
            }

            #[test]
            fn test_input_explicit_positive() {
                assert_input("+3.14", "N", "N!", 3.14_f32);
            }
        }

        mod string_var {
            use crate::interpreter::test_utils::*;

            #[test]
            fn test_input_hello() {
                assert_input("hello", "A$", "A$", "hello");
            }

            #[test]
            fn test_input_does_not_trim_new_line() {
                assert_input("hello\r\n", "A$", "A$", "hello\r\n");
            }
        }

        mod int_var {
            use crate::interpreter::test_utils::*;

            #[test]
            fn test_input_42() {
                assert_input("42", "A%", "A%", 42);
            }
        }
    }

    #[test]
    fn test_sub_call_environ() {
        let program = r#"
        ENVIRON "FOO=BAR"
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.get_env_var(&"FOO".to_string()), "BAR");
    }

    #[test]
    fn test_interpret_sub_call_user_defined_no_args() {
        let program = r#"
        DECLARE SUB Hello
        Hello
        SUB Hello
            ENVIRON "FOO=BAR"
        END SUB
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.get_env_var(&"FOO".to_string()), "BAR");
    }

    #[test]
    fn test_interpret_sub_call_user_defined_two_args() {
        let program = r#"
        DECLARE SUB Hello(N$, V$)
        Hello "FOO", "BAR"
        SUB Hello(N$, V$)
            ENVIRON N$ + "=" + V$
        END SUB
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.get_env_var(&"FOO".to_string()), "BAR");
    }

    #[test]
    fn test_interpret_sub_call_user_defined_literal_arg() {
        let program = r#"
        DECLARE SUB Hello(X)
        A = 1
        Hello 5
        PRINT A
        SUB Hello(X)
            X = 42
        END SUB
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.output, vec!["1"]);
    }

    #[test]
    fn test_interpret_sub_call_user_defined_var_arg_is_by_ref() {
        let program = r#"
        DECLARE SUB Hello(X)
        A = 1
        Hello A
        PRINT A
        SUB Hello(X)
            X = 42
        END SUB
        "#;
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.output, vec!["42"]);
    }

    #[test]
    fn test_interpret_sub_call_user_defined_cannot_access_global_scope() {
        let program = "
        DECLARE SUB Hello
        A = 1
        Hello
        PRINT A
        SUB Hello
            A = 42
        END SUB
        ";
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.output, vec!["1"]);
    }

    #[test]
    fn test_stacktrace() {
        let program = r#"
        DECLARE SUB Hello(N)

        Hello 1

        SUB Hello(N)
            If N <= 1 THEN
                Hello N + 1
            ELSE
                Environ "oops"
            END IF
        END SUB
        "#;
        assert_eq!(
            interpret_err(program),
            InterpreterError::new(
                "Invalid expression. Must be name=value.",
                vec![
                    Location::new(10, 17), // "inside" Environ
                    Location::new(10, 17), // at Environ "oops"
                    Location::new(8, 17),  // at Hello N + 1
                    Location::new(4, 9),   // at Hello 1
                ]
            )
        );
    }

    #[test]
    fn test_cannot_override_built_in_sub_with_declaration() {
        let program = r#"
        DECLARE SUB Environ
        PRINT "Hello"
        SUB Environ
        END SUB
        "#;
        assert_linter_err!(program, LinterError::DuplicateDefinition, 4, 9);
    }

    #[test]
    fn test_cannot_override_built_in_sub_without_declaration() {
        let program = r#"
        PRINT "Hello"
        SUB Environ
        END SUB
        "#;
        assert_linter_err!(program, LinterError::DuplicateDefinition, 3, 9);
    }

    #[test]
    fn test_by_ref_parameter_type_mismatch() {
        let program = "
        DECLARE SUB Hello(N)
        A% = 42
        Hello A%
        SUB Hello(N)
            N = N + 1
        END SUB
        ";
        assert_linter_err!(program, LinterError::ArgumentTypeMismatch, 4, 15);
    }

    #[test]
    fn test_by_ref_parameter_const_is_ok_does_not_modify_const() {
        let program = "
        DECLARE SUB Hello(N)
        CONST A = 42
        Hello A%
        PRINT A
        SUB Hello(N)
            PRINT N
            N = N + 1
            PRINT N
        END SUB
        ";
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.output, vec!["42", "43", "42"]);
    }

    #[test]
    fn test_by_val_parameter_cast() {
        let program = "
        DECLARE SUB Hello(N%)
        Hello 3.14
        SUB Hello(N%)
            PRINT N%
            N% = N% + 1
            PRINT N%
        END SUB
        ";
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.output, vec!["3", "4"]);
    }

    #[test]
    fn test_by_ref_parameter_defined_in_previous_sub_call() {
        let program = "
        DECLARE SUB Add(N%)
        INPUT N%
        PRINT N%
        Add N%
        PRINT N%
        SUB Add(N%)
            N% = N% + 1
        END SUB
        ";
        let mut stdlib = MockStdlib::new();
        stdlib.add_next_input("42");
        let interpreter = interpret_with_stdlib(program, stdlib);
        assert_eq!(interpreter.stdlib.output, vec!["42", "43"]);
    }

    #[test]
    fn test_by_ref_two_levels_deep() {
        let program = "
        N = 41
        Sub1 N
        PRINT N

        SUB Sub1(N)
            Sub2 N, 1
        END SUB

        SUB Sub2(N, P)
            N = N + P
        END SUB
        ";
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.output, vec!["42"]);
    }

    #[test]
    fn test_by_ref_two_levels_deep_referencing_parent_constant() {
        let program = "
        Sub1 N
        PRINT N

        SUB Sub1(N)
            Sub2 A
            N = A
        END SUB

        SUB Sub2(A)
            A = 42
        END SUB
        ";
        let interpreter = interpret(program);
        assert_eq!(interpreter.stdlib.output, vec!["42"]);
    }
}
