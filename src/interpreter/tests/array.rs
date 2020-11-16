use crate::assert_interpreter_err;
use crate::assert_prints;
use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;

#[test]
fn test_assign_one_element() {
    let input = r#"
    DIM A(1 TO 1)
    A(1) = 42
    PRINT A(1)
    "#;
    assert_prints!(input, "42");
}

#[test]
fn test_usage_on_global_scope() {
    let input = r#"
    DIM A(3)
    FOR I% = 0 TO 3
        A(I%) = I% + 10
        PRINT A(I%)
    NEXT
    "#;
    assert_prints!(input, "10", "11", "12", "13");
}

#[test]
fn test_usage_on_global_scope_casting_float_indices_to_int() {
    let input = r#"
    DIM A(3)
    FOR I = 0 TO 3
        A(I) = I
        PRINT A(I)
    NEXT
    "#;
    assert_prints!(input, "0", "1", "2", "3");
}

#[test]
fn test_subscript_out_of_range_low_bound() {
    let input = r#"
    DIM A(1 TO 3)
    PRINT A(0)
    "#;
    assert_interpreter_err!(input, QError::SubscriptOutOfRange, 3, 11);
}

#[test]
fn test_subscript_out_of_range_upper_bound() {
    let input = r#"
    DIM A(3)
    PRINT A(4)
    "#;
    assert_interpreter_err!(input, QError::SubscriptOutOfRange, 3, 11);
}

#[test]
fn test_two_dimensional() {
    let input = r#"
    DIM A(1 TO 2, 1 TO 3)
    FOR R = 1 TO 2
        FOR C = 1 TO 3
            A(R, C) = R * 10 + C
            PRINT A(R, C)
        NEXT
    NEXT
    "#;
    assert_prints!(input, "11", "12", "13", "21", "22", "23");
}

#[test]
fn test_parameter() {
    let input = r#"
    DIM choice$(1 TO 3)
    choice$(1) = "Red"
    choice$(2) = "Green"
    choice$(3) = "Blue"

    Menu choice$(), 1, 3

    SUB Menu(choice$(), start%, stop%)
        FOR I% = start% TO stop%
            PRINT choice$(I%)
        NEXT
    END SUB
    "#;
    assert_prints!(input, "Red", "Green", "Blue");
}

#[test]
fn test_parameter_with_lbound_ubound() {
    let input = r#"
    DIM choice$(1 TO 3)
    choice$(1) = "Red"
    choice$(2) = "Green"
    choice$(3) = "Blue"

    Menu choice$()

    SUB Menu(choice$())
        FOR I = LBOUND(choice$) TO UBOUND(choice$)
            PRINT choice$(I)
        NEXT I
    END SUB
    "#;
    assert_prints!(input, "Red", "Green", "Blue");
}

#[test]
fn test_modify_element_in_sub() {
    let input = r#"
    DIM choice$(1 TO 3)
    choice$(1) = "Red"
    choice$(2) = "Green"
    choice$(3) = "Blue"

    Rotate choice$()
    FOR I = LBOUND(choice$) TO UBOUND(choice$)
        PRINT choice$(I)
    NEXT I

    SUB Rotate(choice$())
        DIM first AS STRING
        first = choice$(LBOUND(choice$))
        FOR I = LBOUND(choice$) TO UBOUND(choice$) - 1
            choice$(I) = choice$(I + 1)
        NEXT I
        choice$(UBOUND(choice$)) = first
    END SUB
    "#;
    assert_prints!(input, "Green", "Blue", "Red");
}
