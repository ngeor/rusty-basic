use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn test_for_loop_with_wrong_next_counter() {
    let input = "
            FOR i% = 1 TO 5
                PRINT i%
            NEXT i
            ";
    assert_linter_err!(input, QError::NextWithoutFor, 4, 18);
}

#[test]
fn test_for_loop_with_string_variable() {
    let input = "
            FOR a$ = 1 TO 5
            NEXT
            ";
    assert_linter_err!(input, QError::TypeMismatch, 2, 13);
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
    assert_linter_err!(input, QError::DotClash, 5, 13);
}
