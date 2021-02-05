use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;

#[test]
fn print_local_const_with_dot_in_name() {
    let program = "
    CONST A.B = 42
    PRINT A.B
    ";
    assert_prints!(program, "42");
}

#[test]
fn print_parent_const_with_dot_in_name() {
    let program = "
    CONST A.B = 42
    Foo
    SUB Foo
        PRINT A.B
    END SUB
    ";
    assert_prints!(program, "42");
}
