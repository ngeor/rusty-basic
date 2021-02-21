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
fn on_error_must_use_global_label() {
    let input = r#"
    SUB Test
        ON ERROR GOTO ErrTrap
        EXIT SUB
        ErrTrap:
            RESUME NEXT
    END SUB
    "#;
    assert_linter_err!(input, QError::LabelNotDefined, 3, 9);
}
