use crate::assert_interpreter_err;
use crate::assert_prints;
use crate::assert_prints_exact;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::*;

#[test]
fn go_sub() {
    let input = r#"
    FOR i% = 1 TO 3
        GOSUB Square
    NEXT i%
    END

    Square:
    PRINT i%; i% * i%
    RETURN
    "#;
    assert_prints_exact!(input, " 1  1 ", " 2  4 ", " 3  9 ", "");
}

#[test]
fn go_sub_inside_sub() {
    let input = r#"
    Test

    SUB Test
        i% = 1
        GOSUB Alpha
        EXIT SUB
    Alpha:
        PRINT i%
        RETURN
    END SUB
    "#;
    assert_prints!(input, "1");
}

#[test]
fn go_sub_return_to_specific_address() {
    let input = r#"
    PRINT "hi"
    GOSUB Alpha
    PRINT "invisible"

    Beta:
    PRINT "bye"
    END

    Alpha:
    PRINT "alpha"
    RETURN Beta
    "#;
    assert_prints!(input, "hi", "alpha", "bye");
}

#[test]
fn go_sub_without_return() {
    let input = r#"
    PRINT "hi"
    GOSUB Alpha
    PRINT "invisible"

    Alpha:
    PRINT "bye"
    "#;
    assert_prints!(input, "hi", "bye");
}

#[test]
fn return_without_go_sub() {
    let input = r#"
    RETURN Alpha
    Alpha:
    PRINT "hi"
    "#;
    assert_interpreter_err!(input, QError::ReturnWithoutGoSub, 2, 5);
}

#[test]
fn variable_assigned_in_go_sub_before_definition() {
    let input = r#"
    GOSUB Alpha
    PRINT A
    END

    Alpha:
    A = 42
    RETURN
    "#;
    assert_prints!(input, "42");
}
