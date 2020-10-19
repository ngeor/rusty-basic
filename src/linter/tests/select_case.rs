use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn test_select_wrong_type_in_simple_case() {
    let input = r#"
        SELECT CASE 42
            CASE "book"
                PRINT "hi"
        END SELECT
        "#;
    assert_linter_err!(input, QError::TypeMismatch, 3, 18);
}

#[test]
fn test_select_wrong_type_in_range_case_upper() {
    let input = r#"
        SELECT CASE 42
            CASE 1 TO "book"
                PRINT "hi"
        END SELECT
        "#;
    assert_linter_err!(input, QError::TypeMismatch, 3, 23);
}

#[test]
fn test_select_wrong_type_in_range_case_lower() {
    let input = r#"
        SELECT CASE 42
            CASE "abc" TO 12
                PRINT "hi"
        END SELECT
        "#;
    assert_linter_err!(input, QError::TypeMismatch, 3, 18);
}

#[test]
fn test_select_wrong_type_in_range_case_both() {
    let input = r#"
        SELECT CASE 42
            CASE "abc" TO "def"
                PRINT "hi"
        END SELECT
        "#;
    assert_linter_err!(input, QError::TypeMismatch, 3, 18);
}

#[test]
fn test_select_wrong_type_in_is() {
    let input = r#"
        SELECT CASE 42
            CASE IS >= "abc"
                PRINT "hi"
        END SELECT
        "#;
    assert_linter_err!(input, QError::TypeMismatch, 3, 24);
}
