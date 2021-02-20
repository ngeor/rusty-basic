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

#[test]
fn resume_no_args() {
    let input = r#"
    ON ERROR GOTO ErrHandler
    A = 6
    B = 0
    PRINT A / B
    END

    ErrHandler:
        B = 2
        RESUME
    "#;
    assert_prints!(input, "3");
}

#[test]
fn resume_next() {
    let input = r#"
    ON ERROR GOTO ErrHandler
    A = 6
    B = 0
    PRINT A / B
    PRINT "bye"
    END

    ErrHandler:
        PRINT "error handler invoked"
        RESUME NEXT
    "#;
    assert_prints!(input, "error handler invoked", "bye");
}

#[test]
fn resume_label() {
    let input = r#"
    ON ERROR GOTO ErrHandler
    A = 6
    B = 0
    PRINT A / B
    END

    ErrHandler:
        B = 2
        RESUME Safety

    Safety:
        PRINT "saved by the bell"
    "#;
    assert_prints!(input, "saved by the bell");
}

#[test]
fn on_error_resume_next() {
    let input = r#"
    ON ERROR RESUME NEXT
    PRINT "hello"
    PRINT 1 / 0
    PRINT "bye"
    "#;
    assert_prints!(input, "hello", "bye");
}

#[test]
fn global_error_handler_error_inside_function() {
    let input = r#"
    ON ERROR GOTO ErrTrap
    PRINT "hi"
    A = Division
    PRINT "bye"
    PRINT A
    END

    ErrTrap:
        PRINT "oops"
        RESUME NEXT

    FUNCTION Division
        Division = 42
        PRINT "in division"
        Division = 1 / 0
        PRINT "out of division"
    END FUNCTION
    "#;
    assert_prints!(
        input,
        "hi",
        "in division",
        "oops",
        "out of division",
        "bye",
        "42"
    );
}

#[test]
fn global_error_handler_has_access_to_variables() {
    let input = r#"
    ON ERROR GOTO ErrTrap
    A = 100
    DivisionByZero
    PRINT A
    END

    ErrTrap:
        A = A + 1
        RESUME NEXT

    SUB DivisionByZero
        X = 1 / 0
    END SUB
    "#;
    assert_prints!(input, "101");
}
