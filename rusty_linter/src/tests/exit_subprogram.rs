use crate::assert_linter_err;
use crate::LintError;

#[test]
fn exit_function_not_allowed_in_global_module() {
    let input = "
    EXIT FUNCTION
    ";
    assert_linter_err!(input, LintError::IllegalOutsideSubFunction);
}

#[test]
fn exit_sub_not_allowed_in_function() {
    let input = "
    FUNCTION Hello
    EXIT SUB
    END FUNCTION
    ";
    assert_linter_err!(input, LintError::IllegalInSubFunction);
}

#[test]
fn exit_sub_not_allowed_in_global_module() {
    let input = "
    EXIT SUB
    ";
    assert_linter_err!(input, LintError::IllegalOutsideSubFunction);
}

#[test]
fn exit_function_not_allowed_in_sub() {
    let input = "
    SUB Hello
    EXIT FUNCTION
    END SUB
    ";
    assert_linter_err!(input, LintError::IllegalInSubFunction);
}
