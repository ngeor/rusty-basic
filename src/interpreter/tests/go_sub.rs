use crate::assert_prints;
use crate::assert_prints_exact;
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
