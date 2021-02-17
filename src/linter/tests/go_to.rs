use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn on_error_go_to_missing_label() {
    let input = r#"
    ON ERROR GOTO ErrTrap
    "#;
    assert_linter_err!(input, QError::LabelNotDefined, 2, 5);
}

#[test]
fn go_to_missing_label() {
    let input = "
    GOTO Jump
    ";
    assert_linter_err!(input, QError::LabelNotDefined, 2, 5);
}

#[test]
fn go_to_label_in_different_scope_not_allowed() {
    let input = r#"
    GOTO Alpha

    SUB Test
    Alpha:
    PRINT "hi"
    END SUB
    "#;
    assert_linter_err!(input, QError::LabelNotDefined, 2, 5);
}
