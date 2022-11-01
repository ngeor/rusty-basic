use crate::assert_linter_err;
use rusty_common::*;

#[test]
fn resume_missing_label() {
    let input = "
    RESUME Jump
    ";
    assert_linter_err!(input, QError::LabelNotDefined, 2, 5);
}

#[test]
fn resume_illegal_in_function() {
    let input = r#"
    FUNCTION Hi
        RESUME
    END FUNCTION
    "#;
    assert_linter_err!(input, QError::IllegalInSubFunction, 3, 9);
}

#[test]
fn resume_illegal_in_sub() {
    let input = r#"
    SUB Hi
        RESUME
    END SUB
    "#;
    assert_linter_err!(input, QError::IllegalInSubFunction, 3, 9);
}