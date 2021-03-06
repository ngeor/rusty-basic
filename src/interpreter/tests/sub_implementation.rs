use crate::assert_prints;
use crate::interpreter::interpreter_trait::InterpreterTrait;

#[test]
fn test_sub_params_same_name_different_qualifier() {
    let program = r#"
    Hello 42, "answer"
    SUB Hello(A%, A$)
        PRINT A%
        PRINT A$
    END SUB
    "#;
    assert_prints!(program, "42", "answer");
}

#[test]
fn test_sub_param_expression_different_qualifier() {
    let program = r#"
    Hello "answer"
    SUB Hello(A$)
        A% = 42
        PRINT A%
    END SUB
    "#;
    assert_prints!(program, "42");
}

#[test]
fn test_sub_param_same_as_other_function_allowed() {
    let program = r#"
    Hello 2
    SUB Hello(Foo)
        PRINT Foo + Foo(Foo)
    END SUB
    FUNCTION Foo(Foo)
        Foo = Foo + 1
    END FUNCTION
    "#;
    assert_prints!(program, "5");
}

#[test]
fn possible_to_have_declare_sub_without_implementation() {
    // this is happening on MONEY.BAS, two SUBs are declared but they are not
    // implemented (and they're never called either, which would have been a
    // problem)
    let program = r#"
    DECLARE SUB MakeBackup()
    PRINT "hello"
    "#;
    assert_prints!(program, "hello");
}
