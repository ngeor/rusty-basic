use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;

#[test]
fn test_function_param_same_as_function_name_allowed() {
    let program = r#"
    PRINT Adding(41)
    FUNCTION Adding(Adding)
    Adding = Adding + 1
    END FUNCTION
    "#;
    assert_prints!(program, "42");
}

#[test]
fn test_function_param_same_as_function_name_compact_single_allowed() {
    let program = r#"
    PRINT Adding(41)
    FUNCTION Adding(Adding!)
    Adding = Adding + 1
    END FUNCTION
    "#;
    assert_prints!(program, "42");
}

#[test]
fn test_function_param_same_as_other_function_allowed() {
    let program = r#"
    PRINT Bar(2)

    FUNCTION Foo(Foo)
        Foo = Foo + 1
    END FUNCTION

    FUNCTION Bar(Foo)
        Bar = Foo + Foo(Foo)
    END FUNCTION
    "#;
    assert_prints!(program, "5");
}
