use crate::assert_linter_err;
use rusty_common::*;

#[test]
fn test_for_loop_with_wrong_next_counter() {
    let input = "
    FOR i% = 1 TO 5
        PRINT i%
    NEXT i
    ";
    assert_linter_err!(input, QError::NextWithoutFor, 4, 10);
}

#[test]
fn test_for_loop_with_string_variable() {
    let input = "
    FOR a$ = 1 TO 5
    NEXT
    ";
    assert_linter_err!(input, QError::TypeMismatch, 2, 5);
}

#[test]
fn test_for_loop_var_name_cannot_include_period_if_variable_of_user_defined_type_exists() {
    let input = r#"
    TYPE Card
        Value AS INTEGER
    END TYPE
    FOR A.B = 1 TO 5
    NEXT
    DIM A AS Card
    "#;
    assert_linter_err!(input, QError::DotClash, 5, 9);
}

#[test]
fn do_loop_condition_cannot_be_string() {
    let input = r#"
    DO WHILE A$
    LOOP
    "#;
    assert_linter_err!(input, QError::TypeMismatch, 2, 14);
}

#[test]
fn while_condition_cannot_be_string() {
    let input = r#"
    WHILE A$
    WEND
    "#;
    assert_linter_err!(input, QError::TypeMismatch, 2, 11);
}
