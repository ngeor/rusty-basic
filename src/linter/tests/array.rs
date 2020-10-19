use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn test_passing_array_parameter_without_parenthesis() {
    let input = r#"
    DIM choice$(1 TO 3)

    Menu choice$

    SUB Menu(choice$())
    END SUB
    "#;

    assert_linter_err!(input, QError::TypeMismatch, 4, 10);
}
