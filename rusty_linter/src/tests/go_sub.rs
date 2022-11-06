use crate::assert_linter_err;
use crate::LintError;

#[test]
fn return_with_explicit_label_is_illegal_in_sub() {
    let input = r#"
    SUB Hello

    Alpha:
    PRINT "hi"

    RETURN Alpha

    END SUB
    "#;
    assert_linter_err!(input, LintError::IllegalInSubFunction, 7, 5);
}

#[test]
fn go_sub_missing_label() {
    let input = r#"
    GOSUB Alpha
    "#;
    assert_linter_err!(input, LintError::LabelNotDefined, 2, 5);
}

#[test]
fn return_missing_label() {
    let input = r#"
    RETURN Alpha
    "#;
    assert_linter_err!(input, LintError::LabelNotDefined, 2, 5);
}
