use crate::assert_has_variable;
use crate::assert_interpreter_err;
use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::test_utils::*;
use crate::RuntimeError;

macro_rules! assert_assign_ok {
    ($program:expr, $expected_variable_name:expr, $expected_value:expr) => {
        let interpreter = interpret($program);
        let name = rusty_parser::Name::from($expected_variable_name);
        assert_eq!(
            interpreter.context().get_by_name(&name),
            rusty_variant::Variant::from($expected_value)
        );
    };
}

#[test]
fn test_literals() {
    assert_has_variable!(interpret("X = 83.14"), "X!", 83.14_f32);
    assert_has_variable!(interpret("X# = 4.14"), "X#", 4.14);
    assert_has_variable!(interpret("X$ = \"hello\""), "X$", "hello");
    assert_has_variable!(interpret("X% = 42"), "X%", 42);
    assert_has_variable!(interpret("X& = 42"), "X&", 42_i64);
}

#[test]
fn test_assign_literal_to_unqualified_float() {
    assert_assign_ok!("X = 1.0", "X!", 1.0_f32);
    assert_assign_ok!("X = -1.0", "X!", -1.0_f32);
    assert_assign_ok!("X = .5", "X!", 0.5_f32);
    assert_assign_ok!("X = -.5", "X!", -0.5_f32);
    assert_assign_ok!("X = 1", "X!", 1.0_f32);
    assert_assign_ok!("X = 6.14#", "X!", 6.14_f32);
}

#[test]
fn test_assign_plus_expression_to_unqualified_float() {
    assert_assign_ok!("X = .5 + .5", "X!", 1.0_f32);
}

#[test]
fn test_assign_literal_to_qualified_float() {
    assert_assign_ok!("X! = 1.0", "X!", 1.0_f32);
    assert_assign_ok!("X! = 1", "X!", 1.0_f32);
}

#[test]
fn test_assign_literal_to_qualified_double() {
    assert_assign_ok!("X# = 1.0", "X#", 1.0_f64);
    assert_assign_ok!("X# = 1", "X#", 1.0_f64);
    assert_assign_ok!("X# = 2.14#", "X#", 2.14_f64);
}

#[test]
fn test_assign_literal_to_qualified_string() {
    assert_assign_ok!("A$ = \"hello\"", "A$", "hello");
}

#[test]
fn test_assign_literal_to_qualified_integer() {
    assert_assign_ok!("X% = 1.0", "X%", 1);
    assert_assign_ok!("X% = 1.1", "X%", 1);
    assert_assign_ok!("X% = 1.5", "X%", 2);
    assert_assign_ok!("X% = 1.9", "X%", 2);
    assert_assign_ok!("X% = 1", "X%", 1);
    assert_assign_ok!("X% = -1", "X%", -1);
    assert_assign_ok!("X% = 3.14#", "X%", 3);
}

#[test]
fn test_assign_literal_to_qualified_long() {
    assert_assign_ok!("X& = 1.0", "X&", 1_i64);
    assert_assign_ok!("X& = 1.1", "X&", 1_i64);
    assert_assign_ok!("X& = 1.5", "X&", 2_i64);
    assert_assign_ok!("X& = 1.9", "X&", 2_i64);
    assert_assign_ok!("X& = 1", "X&", 1_i64);
    assert_assign_ok!("X& = -1", "X&", -1_i64);
    assert_assign_ok!("X& = 3.14#", "X&", 3_i64);
}

#[test]
fn test_assign_same_variable_name_different_qualifiers() {
    let input = "A = 0.1
    A# = 7.14
    A$ = \"Hello\"
    A% = 1
    A& = 100";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", 0.1_f32);
    assert_has_variable!(interpreter, "A#", 7.14);
    assert_has_variable!(interpreter, "A$", "Hello");
    assert_has_variable!(interpreter, "A%", 1);
    assert_has_variable!(interpreter, "A&", 100_i64);
}

#[test]
fn test_assign_negated_variable() {
    let input = "A = -42
    B = -A";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", -42.0_f32);
    assert_has_variable!(interpreter, "B!", 42.0_f32);
}

#[test]
fn test_assign_variable_bare_lower_case() {
    let input = "
    A = 42
    b = 12
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", 42.0_f32);
    assert_has_variable!(interpreter, "a!", 42.0_f32);
    assert_has_variable!(interpreter, "B!", 12.0_f32);
    assert_has_variable!(interpreter, "b!", 12.0_f32);
}

#[test]
fn test_assign_variable_typed_lower_case() {
    let input = "
    A% = 42
    b% = 12
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A%", 42);
    assert_has_variable!(interpreter, "a%", 42);
    assert_has_variable!(interpreter, "B%", 12);
    assert_has_variable!(interpreter, "b%", 12);
}

#[test]
fn test_increment_variable_bare_lower_case() {
    let input = "
    A = 42
    A = a + 1
    b = 12
    B = b + 1
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", 43_f32);
    assert_has_variable!(interpreter, "a!", 43_f32);
    assert_has_variable!(interpreter, "B!", 13_f32);
    assert_has_variable!(interpreter, "b!", 13_f32);
}

#[test]
fn test_increment_variable_typed_lower_case() {
    let input = "
    A% = 42
    A% = a% + 1
    b% = 12
    B% = b% + 1
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A%", 43);
    assert_has_variable!(interpreter, "a%", 43);
    assert_has_variable!(interpreter, "B%", 13);
    assert_has_variable!(interpreter, "b%", 13);
}

#[test]
fn test_assign_with_def_dbl() {
    let input = "
    DEFDBL A-Z
    A = 9.28
    A! = 8.14
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", 8.14_f32);
    assert_has_variable!(interpreter, "A#", 9.28_f64);
}

#[test]
fn test_assign_with_def_int() {
    let input = "
    DEFINT A-Z
    A = 42
    A! = 5.14
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", 5.14_f32);
    assert_has_variable!(interpreter, "A%", 42);
}

#[test]
fn test_assign_with_def_lng() {
    let input = "
    DEFLNG A-Z
    A = 42
    A! = 1.14
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", 1.14_f32);
    assert_has_variable!(interpreter, "A&", 42_i64);
}

#[test]
fn test_assign_with_def_sng() {
    let input = "
    DEFSNG A-Z
    A = 42
    A! = 2.14
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", 2.14_f32);
}

#[test]
fn test_assign_with_def_str() {
    let input = r#"
    DEFSTR A-Z
    A = "hello"
    A! = 4.14
    "#;
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "A!", 4.14_f32);
    assert_has_variable!(interpreter, "A$", "hello");
}

#[test]
fn test_assign_integer_overflow() {
    assert_assign_ok!("A% = 32767", "A%", 32767_i32);
    assert_interpreter_err!("A% = 32768", RuntimeError::Overflow, 1, 6);
    assert_assign_ok!("A% = -32768", "A%", -32768_i32);
    assert_interpreter_err!("A% = -32769", RuntimeError::Overflow, 1, 6);
}

#[test]
fn test_assign_long_overflow_ok() {
    assert_assign_ok!("A& = 2147483647", "A&", 2147483647_i64);
    assert_assign_ok!("A& = -2147483648", "A&", -2147483648_i64);
}

#[test]
fn test_assign_long_overflow_err() {
    assert_interpreter_err!("A& = 2147483648", RuntimeError::Overflow, 1, 6);
    assert_interpreter_err!("A& = -2147483649", RuntimeError::Overflow, 1, 6);
}

#[test]
fn test_same_variable_name_different_qualifiers() {
    let program = r#"
    A$ = "hello"
    A% = 42
    PRINT A$
    PRINT A%
    "#;
    assert_prints!(program, "hello", "42");
}

#[test]
fn test_can_assign_to_parameter_hiding_name_of_function() {
    let program = r#"
    Hello 41
    FUNCTION Foo
    END FUNCTION

    SUB Hello(Foo)
    Foo = Foo + 1
    PRINT Foo
    END SUB
    "#;
    assert_prints!(program, "42");
}

#[test]
fn test_assign_math_expr() {
    let program = r#"
    R = 3
    C = 4
    A = R * 10 + C
    PRINT A
    "#;
    assert_prints!(program, "34");
}
