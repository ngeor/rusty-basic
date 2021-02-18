use crate::assert_interpreter_err;
use crate::assert_prints;
use crate::assert_prints_exact;
use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;

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
    // TODO rewrite this test program to use EXIT SUB when EXIT SUB is implemented
    let input = r#"
    Test

    SUB Test
        i% = 1
        GOSUB Alpha
        IF i% > 1 THEN
        Alpha:
            PRINT i%
            RETURN
        END IF
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
