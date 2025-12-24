use crate::assert_linter_err;
use crate::core::LintError;

#[test]
fn test_duplicate_global_label() {
    let input = r#"
    PRINT "hi"
    Alpha:
    PRINT "alpha"
    Alpha:
    PRINT "beta"
    "#;

    assert_linter_err!(input, LintError::DuplicateLabel, 5, 5);
}

#[test]
fn test_duplicate_label_in_sub() {
    let input = r#"
    SUB Hello
        PRINT "hi"
        Alpha:
        PRINT "alpha"
        Alpha:
        PRINT "beta"
    END SUB
    "#;

    assert_linter_err!(input, LintError::DuplicateLabel, 6, 9);
}

#[test]
fn test_duplicate_label_in_function() {
    let input = r#"
    FUNCTION Hello
        PRINT "hi"
        Alpha:
        PRINT "alpha"
        Alpha:
        PRINT "beta"
    END FUNCTION
    "#;

    assert_linter_err!(input, LintError::DuplicateLabel, 6, 9);
}

#[test]
fn test_duplicate_label_global_and_sub() {
    let input = r#"
    PRINT "hi"
    Alpha:
    PRINT "bye"

    SUB Hello
        PRINT "hi"
        Alpha:
        PRINT "alpha"
    END SUB
    "#;

    assert_linter_err!(input, LintError::DuplicateLabel, 8, 9);
}

#[test]
fn test_duplicate_label_sub_and_sub() {
    let input = r#"
    SUB Hello
        PRINT "hi"
        Alpha:
        PRINT "alpha"
    END SUB

    SUB Hello2
        PRINT "hi"
        Alpha:
        PRINT "alpha"
    END SUB
    "#;

    assert_linter_err!(input, LintError::DuplicateLabel, 10, 9);
}
