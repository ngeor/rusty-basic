use crate::assert_prints;
use crate::common::*;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::test_utils::*;
use crate::interpreter::Stdlib;

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
    assert_eq!(interpreter.stdlib().get_env_var(&"FOO".to_string()), "BAR");
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
    assert_eq!(interpreter.stdlib().get_env_var(&"FOO".to_string()), "BAR");
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
    assert_prints!(program, "1");
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
    assert_prints!(program, "42");
}

#[test]
fn test_interpret_sub_call_var_in_parenthesis_is_by_val() {
    let program = r#"
    DECLARE SUB Hello(X)
    A = 1
    Hello (A)
    PRINT A
    SUB Hello(X)
        PRINT X
        X = 42
        PRINT X
    END SUB
    "#;
    assert_prints!(program, "1", "42", "1");
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
    assert_prints!(program, "1");
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
        ErrorEnvelope::Stacktrace(
            QError::Other("Invalid expression. Must be name=value.".to_string()),
            vec![
                Location::new(10, 13), // "inside" Environ
                Location::new(8, 13),  // at Hello N + 1
                Location::new(4, 5),   // at Hello 1
            ]
        )
    );
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
    assert_prints!(program, "42", "43", "42");
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
    assert_prints!(program, "3", "4");
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
    let mut interpreter = interpret_with_raw_input(program, "42");
    assert_eq!(interpreter.stdout().output_lines(), vec!["42", "43"]);
}

#[test]
fn test_by_ref_two_levels_deep() {
    let program = r#"
    N = 41
    Sub1 N
    PRINT N

    SUB Sub1(N)
        PRINT "Begin Sub1", N
        Sub2 N, 1
        PRINT "End Sub1", N
    END SUB

    SUB Sub2(N, P)
        PRINT "Begin Sub2", N
        N = N + P
        PRINT "End Sub2", N
    END SUB
    "#;
    assert_prints!(
        program,
        "Begin Sub1     41",
        "Begin Sub2     41",
        "End Sub2       42",
        "End Sub1       42",
        "42"
    );
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
    assert_prints!(program, "42");
}

#[test]
fn test_sub_call_parenthesis() {
    let program = "
    Hello(1)

    SUB Hello(N)
        PRINT N
    END SUB
    ";
    assert_prints!(program, "1");
}

#[test]
fn test_dot_in_sub_declaration_name() {
    let program = r#"
    DECLARE SUB Hello.World

    Hello.World

    SUB Hello.World
        PRINT "Hello, world!"
    END SUB
    "#;
    assert_prints!(program, "Hello, world!");
}

#[test]
fn test_dot_in_sub_param_name() {
    let program = r#"
    Hello.World "Hello there"

    SUB Hello.World (greet.msg$)
        PRINT greet.msg$
    END SUB
    "#;
    assert_prints!(program, "Hello there");
}

mod static_sub {
    use super::*;

    #[test]
    fn static_sub_preserves_values_between_calls() {
        let program = r#"
        Hello
        Hello

        SUB Hello STATIC
            A = A + 1
            PRINT A
        END SUB
        "#;
        assert_prints!(program, "1", "2");
    }

    #[test]
    fn regular_sub_does_not_preserve_values_between_calls() {
        let program = r#"
        Hello
        Hello

        SUB Hello
            A = A + 1
            PRINT A
        END SUB
        "#;
        assert_prints!(program, "1", "1");
    }

    #[test]
    fn static_sub_with_arguments() {
        let program = r#"
        Hello 1
        Hello 2

        SUB Hello(B) STATIC
            A = A + B
            PRINT A
        END SUB
        "#;
        assert_prints!(program, "1", "3");
    }

    #[test]
    fn fixed_length_string_append() {
        let program = r#"
        Hello
        Hello

        SUB Hello STATIC
            DIM A AS STRING * 5
            A = A + "!"
            PRINT A
        END SUB
        "#;
        assert_prints!(program, "", "");
    }

    #[test]
    fn fixed_length_string_prepend() {
        let program = r#"
        Hello
        Hello

        SUB Hello STATIC
            DIM A AS STRING * 5
            A = "!" + A
            PRINT A
        END SUB
        "#;
        assert_prints!(program, "!", "!!");
    }

    #[test]
    fn user_defined_type() {
        let program = r#"
        TYPE Card
            Value AS INTEGER
        END TYPE
        Hello
        Hello

        SUB Hello STATIC
            DIM A AS Card
            A.Value = A.Value + 1
            PRINT A.Value
        END SUB
        "#;
        assert_prints!(program, "1", "2");
    }

    #[test]
    fn array_built_in() {
        let program = r#"
        Hello
        Hello

        SUB Hello STATIC
            DIM A(2) AS INTEGER
            A(1) = A(1) + 1
            PRINT A(1)
        END SUB
        "#;
        assert_prints!(program, "1", "2");
    }

    #[test]
    fn array_user_defined_type() {
        let program = r#"
        TYPE Card
            Value AS INTEGER
        END TYPE
        Hello
        Hello

        SUB Hello STATIC
            DIM A(2) AS Card
            A(1).Value = A(1).Value + 1
            PRINT A(1).Value
        END SUB
        "#;
        assert_prints!(program, "1", "2");
    }

    #[test]
    fn redim_does_not_preserve_values() {
        let program = r#"
        Hello
        Hello

        SUB Hello STATIC
            REDIM A(1 TO 2)
            PRINT A(1)
            A(1) = A(1) + 1
        END SUB
        "#;
        assert_prints!(program, "0", "0");
    }
}
