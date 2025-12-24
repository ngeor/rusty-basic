use crate::assert_linter_err;
use crate::core::LintError;

#[test]
fn go_to_missing_label() {
    let input = "
    GOTO Jump
    ";
    assert_linter_err!(input, LintError::LabelNotDefined, 2, 5);
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
    assert_linter_err!(input, LintError::LabelNotDefined, 2, 5);
}
