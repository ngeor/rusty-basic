use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;

#[test]
fn exit_function() {
    let input = r#"
    PRINT 1
    A = Hello
    PRINT A

    FUNCTION Hello
        Hello = 42
        EXIT FUNCTION
        Hello = 41
    END FUNCTION
    "#;
    assert_prints!(input, "1", "42");
}

#[test]
fn exit_sub() {
    let input = r#"
    PRINT 1
    Hello
    PRINT 2

    SUB Hello
        PRINT 3
        EXIT SUB
        PRINT 4
    END SUB
    "#;
    assert_prints!(input, "1", "3", "2");
}
