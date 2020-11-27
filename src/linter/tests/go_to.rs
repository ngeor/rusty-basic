use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn on_error_go_to_missing_label() {
    let input = r#"
            ON ERROR GOTO ErrTrap
            "#;
    assert_linter_err!(input, QError::LabelNotDefined, 2, 13);
}

#[test]
fn go_to_missing_label() {
    let input = "
            GOTO Jump
            ";
    assert_linter_err!(input, QError::LabelNotDefined, 2, 13);
}

#[test]
fn go_to_duplicate_label() {
    let input = "
            GOTO Jump
            Jump:
            Jump:
            ";
    assert_linter_err!(input, QError::DuplicateLabel, 4, 13);
}
