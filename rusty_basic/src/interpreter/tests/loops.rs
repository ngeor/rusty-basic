use crate::assert_prints;
use crate::assert_prints_exact;
use crate::assert_prints_nothing;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::test_utils::*;
use crate::RuntimeError;
use crate::{assert_has_variable, ErrorEnvelope};
use rusty_common::*;

#[test]
fn test_simple_for_loop() {
    let names = ["I", "ABC", "A.B.", "I%", "a", "a%", "step2"];
    for name in &names {
        let input = format!(
            "
            FOR {} = 1 TO 5
                PRINT {}
            NEXT
            ",
            name, name
        );
        assert_prints!(&input, "1", "2", "3", "4", "5");
    }
}

#[test]
fn test_simple_for_loop_value_of_variable_after_loop() {
    let input = "
    FOR i% = 1 TO 5
        PRINT i%
    NEXT
    ";
    let interpreter = interpret(input);
    assert_has_variable!(interpreter, "i%", 6);
}

#[test]
fn test_simple_for_loop_value_of_variable_after_loop_never_entering() {
    let input = "
    FOR i% = 1 TO -1
        PRINT i%
    NEXT
    ";
    let mut interpreter = interpret(input);
    assert_has_variable!(interpreter, "i%", 1);
    assert_eq!(interpreter.stdout().output(), "");
}

#[test]
fn test_for_loop_with_positive_step() {
    let input = "
    FOR i% = 1 TO 7 STEP 2
        PRINT i%
    NEXT
    ";
    assert_prints!(input, "1", "3", "5", "7");
}

#[test]
fn test_for_loop_with_negative_step() {
    let input = "
    FOR i% = 7 TO -6 STEP -3
        PRINT i%
    NEXT
    ";
    assert_prints!(input, "7", "4", "1", "-2", "-5");
}

#[test]
fn test_for_loop_with_zero_step() {
    let input = "
    FOR i% = 7 TO -6 STEP 0
        PRINT i%
    NEXT
    ";
    assert_eq!(
        interpret_err(input),
        ErrorEnvelope::Pos(RuntimeError::ForLoopZeroStep, Position::new(2, 27))
    );
}

#[test]
fn test_for_loop_with_negative_step_minus_one() {
    let input = "
    FOR i% = 3 TO -3 STEP -1
        PRINT i%
    NEXT
    ";
    let mut interpreter = interpret(input);
    assert_has_variable!(interpreter, "i%", -4);
    assert_eq!(
        interpreter.stdout().output_lines(),
        vec!["3", "2", "1", "0", "-1", "-2", "-3"]
    );
}

#[test]
fn test_for_loop_with_specified_next_counter() {
    let input = "
    FOR i% = 1 TO 5
        PRINT i%
    NEXT i%
    ";
    assert_prints!(input, "1", "2", "3", "4", "5");
}

#[test]
fn test_for_loop_with_specified_next_counter_lower_case() {
    let input = "
    FOR i% = 1 TO 5
        PRINT i%
    NEXT I%
    ";
    assert_prints!(input, "1", "2", "3", "4", "5");
}

#[test]
fn test_for_loop_end_expression_evaluated_only_once() {
    let input = "
    N% = 3
    FOR I% = 1 TO N%
        PRINT I%
        N% = N% - 1
    NEXT
    ";
    let mut interpreter = interpret(input);
    assert_has_variable!(interpreter, "I%", 4);
    assert_has_variable!(interpreter, "N%", 0);
    assert_eq!(interpreter.stdout().output_lines(), vec!["1", "2", "3"]);
}

#[test]
fn test_nested_for_loop() {
    let input = r#"
    FOR I = 1 to 2
    FOR J = 3 to 4
    PRINT I; " "; J
    NEXT
    NEXT
    "#;
    assert_prints_exact!(input, " 1   3 ", " 1   4 ", " 2   3 ", " 2   4 ", "");
}

#[test]
fn test_for_loop_parenthesis_expressions() {
    let input = "
    FOR I=(1+2)TO(2+3)STEP(1+0)
        PRINT I
    NEXT
    ";
    assert_prints!(input, "3", "4", "5");
}

#[test]
fn test_for_loop_assigning_to_function_name() {
    let input = "
    PRINT ForVar

    FUNCTION ForVar
        FOR ForVar = 1 TO 5
        NEXT
    END FUNCTION
    ";
    assert_prints!(input, "6");
}

#[test]
fn test_while_wend() {
    let input = "
    A = 1
    WHILE A < 5
        PRINT A
        A = A + 1
    WEND
    ";
    assert_prints!(input, "1", "2", "3", "4");
}

#[test]
fn do_while_condition_loop() {
    let input = r#"
    DO WHILE A < 3
        PRINT A
        A = A + 1
    LOOP
    "#;
    assert_prints!(input, "0", "1", "2");
}

#[test]
fn do_until_condition_loop() {
    let input = r#"
    A = 1
    DO UNTIL A > 3
        PRINT A
        A = A + 1
    LOOP
    "#;
    assert_prints!(input, "1", "2", "3");
}

#[test]
fn do_loop_while_condition() {
    let input = r#"
    DO
        PRINT A
        A = A + 1
    LOOP WHILE A < 3
    "#;
    assert_prints!(input, "0", "1", "2");
}

#[test]
fn do_loop_until_condition() {
    let input = r#"
    A = 1
    DO
        PRINT A
        A = A + 1
    LOOP UNTIL A > 3
    "#;
    assert_prints!(input, "1", "2", "3");
}

#[test]
fn do_while_condition_does_not_enter_loop_if_false() {
    let input = r#"
    A = 5
    DO WHILE A < 3
        PRINT A
        A = A + 1
    LOOP
    "#;
    assert_prints_nothing!(input);
}

#[test]
fn do_loop_while_condition_enter_loop_if_false() {
    let input = r#"
    A = 5
    DO
        PRINT A
        A = A + 1
    LOOP WHILE A < 3
    "#;
    assert_prints!(input, "5");
}
