use crate::assert_has_variable;
use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::test_utils::*;

#[test]
fn test_function_call_declared_and_implemented() {
    let program = "
    DECLARE FUNCTION Add(A, B)
    X = Add(1, 2)
    FUNCTION Add(A, B)
        Add = A + B
    END FUNCTION
    ";
    let interpreter = interpret(program);
    assert_has_variable!(interpreter, "X!", 3.0_f32);
}

#[test]
fn test_no_args_function_call() {
    let program = r#"
    DECLARE FUNCTION GetAction$

    PRINT GetAction$

    FUNCTION GetAction$
        GetAction$ = "hello"
    END FUNCTION
    "#;
    assert_prints!(program, "hello");
}

#[test]
fn test_function_call_without_declaration() {
    let program = "
    X = Add(1, 2)
    FUNCTION Add(A, B)
        Add = A + B
    END FUNCTION
    ";
    let interpreter = interpret(program);
    assert_has_variable!(interpreter, "X!", 3.0_f32);
}

#[test]
fn test_function_call_not_setting_return_value_defaults_to_zero() {
    let program = "
    DECLARE FUNCTION Add(A, B)
    X = Add(1, 2)
    FUNCTION Add(A, B)
        PRINT A + B
    END FUNCTION
    ";
    let mut interpreter = interpret(program);
    assert_has_variable!(interpreter, "X!", 0.0_f32);
    assert_eq!(interpreter.stdout().output_lines(), vec!["3"]);
}

#[test]
fn test_function_call_missing_returns_zero() {
    let program = "
    X = Add(1, 2)
    ";
    let interpreter = interpret(program);
    assert_has_variable!(interpreter, "X!", 0.0_f32);
}

#[test]
fn test_function_call_lowercase() {
    let program = "
    DECLARE FUNCTION Add(A, B, c)
    X = add(1, 2, 3)
    FUNCTION ADD(a, B, C)
        aDd = a + b + c
    END FUNCTION
    ";
    let interpreter = interpret(program);
    assert_has_variable!(interpreter, "X!", 6.0_f32);
}

#[test]
fn test_function_call_defint() {
    let program = "
    DEFINT A-Z
    DECLARE FUNCTION Add(A, B, c)
    X = add(1, 2, 3)
    FUNCTION ADD(a, B, C)
        aDd = a + b + c
    END FUNCTION
    ";
    let interpreter = interpret(program);
    assert_has_variable!(interpreter, "X%", 6);
}

#[test]
fn test_function_call_defstr() {
    let program = r#"
    DEFSTR A-Z
    DECLARE FUNCTION Add(A, B, c)
    X = add("1", "2", "3")
    FUNCTION ADD(a, B, C)
        aDd = a + b + c
    END FUNCTION
    "#;
    let interpreter = interpret(program);
    assert_has_variable!(interpreter, "X$", "123");
}

#[test]
fn test_interpret_function_call_user_defined_literal_arg() {
    let program = r#"
    DECLARE FUNCTION Hello(X)
    A = 1
    B = Hello(A + 1)
    PRINT A
    PRINT B
    FUNCTION Hello(X)
        X = X + 1
        Hello = X + 1
    END FUNCTION
    "#;
    assert_prints!(program, "1", "4");
}

#[test]
fn test_interpret_function_call_user_defined_var_arg_is_by_ref() {
    let program = r#"
    DECLARE FUNCTION Hello(X)
    A = 1
    B = Hello(A)
    PRINT A
    PRINT B
    FUNCTION Hello(X)
        X = X + 1
        Hello = X + 1
    END FUNCTION
    "#;
    assert_prints!(program, "2", "3");
}

#[test]
fn test_interpret_function_call_user_defined_var_arg_is_by_ref_assign_to_self() {
    let program = r#"
    DECLARE FUNCTION Hello(X)
    A = 1
    A = Hello(A)
    PRINT A
    FUNCTION Hello(X)
        X = X + 1
        Hello = X + 1
    END FUNCTION
    "#;
    assert_prints!(program, "3");
}

#[test]
fn test_recursive_function() {
    let program = r#"
    DECLARE FUNCTION Sum(X)

    PRINT Sum(3)

    FUNCTION Sum(X)
        IF 1 < X THEN
            Sum = Sum(X - 1) + X
        ELSE
            Sum = 1
        END IF
    END FUNCTION
    "#;
    assert_prints!(program, "6");
}

#[test]
fn test_dot_in_function_name_even_if_it_is_a_keyword() {
    let program = r#"
    DECLARE FUNCTION Upper.Case$
    PRINT Upper.Case
    FUNCTION Upper.Case$
        Upper.Case = "ABC"
    END FUNCTION
    "#;
    assert_prints!(program, "ABC");
}
