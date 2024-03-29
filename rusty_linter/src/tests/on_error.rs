use crate::assert_linter_err;
use crate::LintError;

#[test]
fn on_error_go_to_missing_label() {
    let input = r#"
    ON ERROR GOTO ErrTrap
    "#;
    assert_linter_err!(input, LintError::LabelNotDefined, 2, 5);
}

#[test]
fn on_error_must_use_global_label() {
    let input = r#"
    SUB Test
        ON ERROR GOTO ErrTrap
        EXIT SUB
        ErrTrap:
            SYSTEM
    END SUB
    "#;
    assert_linter_err!(input, LintError::LabelNotDefined, 3, 9);
}
