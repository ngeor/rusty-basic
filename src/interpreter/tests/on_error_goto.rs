use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;

#[test]
fn on_error_go_to_label() {
    let input = r#"
    ON ERROR GOTO ErrTrap
    Environ "ShouldHaveAnEqualsSignInHereSomewhere"
    PRINT "Will not print this"
    SYSTEM
    ErrTrap:
        PRINT "Saved by the bell"
    "#;
    assert_prints!(input, "Saved by the bell");
}

#[test]
fn on_error_go_to_label_with_dots_in_label_name() {
    let input = r#"
    ON ERROR GOTO Err.Trap
    PRINT 1 / 0
    SYSTEM
    Err.Trap:
        PRINT "Almost divided by zero"
    "#;
    assert_prints!(input, "Almost divided by zero");
}
