use crate::assert_linter_err;
use crate::core::LintError;

macro_rules! assert_condition_err {
    ($condition:expr) => {
        let program = format!(
            r#"
            IF {} THEN
                PRINT "hi"
            END IF
            "#,
            $condition
        );
        assert_linter_err!(&program, $crate::core::LintError::TypeMismatch);
    };
}

#[test]
fn test_type_mismatch() {
    assert_linter_err!("X = 1.1 + \"hello\"", LintError::TypeMismatch, 1, 11);
    assert_linter_err!("X = 1.1# + \"hello\"", LintError::TypeMismatch, 1, 12);
    assert_linter_err!("X$ = \"hello\" + 1", LintError::TypeMismatch, 1, 16);
    assert_linter_err!("X$ = \"hello\" + 1.1", LintError::TypeMismatch, 1, 16);
    assert_linter_err!("X$ = \"hello\" + 1.1#", LintError::TypeMismatch, 1, 16);
    assert_linter_err!("X% = 1 + \"hello\"", LintError::TypeMismatch, 1, 10);
    assert_linter_err!("X& = 1 + \"hello\"", LintError::TypeMismatch, 1, 10);
    assert_linter_err!("X = 1.1 - \"hello\"", LintError::TypeMismatch, 1, 11);
    assert_linter_err!("X = 1.1# - \"hello\"", LintError::TypeMismatch, 1, 12);
    assert_linter_err!("X$ = \"hello\" - \"hi\"", LintError::TypeMismatch, 1, 16);
    assert_linter_err!("X$ = \"hello\" - 1", LintError::TypeMismatch, 1, 16);
    assert_linter_err!("X$ = \"hello\" - 1.1", LintError::TypeMismatch, 1, 16);
    assert_linter_err!("X$ = \"hello\" - 1.1#", LintError::TypeMismatch, 1, 16);
    assert_linter_err!("X$ = 1 - \"hello\"", LintError::TypeMismatch, 1, 10);
    assert_linter_err!("X& = 1 - \"hello\"", LintError::TypeMismatch, 1, 10);
    assert_linter_err!(r#"PRINT "hello" * 5"#, LintError::TypeMismatch, 1, 17);
    assert_linter_err!(r#"PRINT "hello" / 5"#, LintError::TypeMismatch, 1, 17);
    assert_linter_err!("X = -\"hello\"", LintError::TypeMismatch, 1, 6);
    assert_linter_err!("X% = -\"hello\"", LintError::TypeMismatch, 1, 7);
    assert_linter_err!("X = NOT \"hello\"", LintError::TypeMismatch, 1, 9);
    assert_linter_err!("X% = NOT \"hello\"", LintError::TypeMismatch, 1, 10);

    assert_linter_err!(r#"PRINT 1 AND "hello""#, LintError::TypeMismatch, 1, 13);
    assert_linter_err!(r#"PRINT "hello" AND 1"#, LintError::TypeMismatch, 1, 19);
    assert_linter_err!(r#"PRINT "hello" AND "bye""#, LintError::TypeMismatch, 1, 19);
}

#[test]
fn test_condition_type_mismatch() {
    assert_condition_err!("9.1 < \"hello\"");
    assert_condition_err!("9.1# < \"hello\"");
    assert_condition_err!("\"hello\" < 3.14");
    assert_condition_err!("\"hello\" < 3");
    assert_condition_err!("\"hello\" < 3.14#");
    assert_condition_err!("9 < \"hello\"");
    assert_condition_err!("9.1 <= \"hello\"");
    assert_condition_err!("9.1# <= \"hello\"");
    assert_condition_err!("\"hello\" <= 3.14");
    assert_condition_err!("\"hello\" <= 3");
    assert_condition_err!("\"hello\" <= 3.14#");
    assert_condition_err!("9 <= \"hello\"");
}

#[test]
fn qualified_const_usage_wrong_type() {
    let program = "
            CONST X = 42
            PRINT X!
            ";
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 19);
}

#[test]
fn test_function_call_expression_no_args() {
    assert_linter_err!("PRINT IsValid()", LintError::FunctionNeedsArguments);
}

#[test]
fn test_function_call_qualified_expression_no_args() {
    assert_linter_err!("PRINT IsValid%()", LintError::FunctionNeedsArguments);
}
