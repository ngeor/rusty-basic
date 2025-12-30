use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::{assert_prints, assert_prints_nothing};

#[test]
fn test_if_block_true() {
    let input = "
    IF 1 < 2 THEN
        PRINT \"hello\"
    END IF
    ";
    assert_prints!(input, "hello");
}

#[test]
fn test_if_block_false() {
    let input = "
    IF 2 < 1 THEN
        PRINT \"hello\"
    END IF
    ";
    assert_prints_nothing!(input);
}

#[test]
fn test_if_else_block_true() {
    let input = "
    IF 1 < 2 THEN
        PRINT \"hello\"
    ELSE
        PRINT \"bye\"
    END IF
    ";
    assert_prints!(input, "hello");
}

#[test]
fn test_if_else_block_false() {
    let input = "
    IF 2 < 1 THEN
        PRINT \"hello\"
    ELSE
        PRINT \"bye\"
    END IF
    ";
    assert_prints!(input, "bye");
}

#[test]
fn test_if_elseif_block_true_true() {
    let input = "
    IF 1 < 2 THEN
        PRINT \"hello\"
    ELSEIF 1 < 2 THEN
        PRINT \"bye\"
    END IF
    ";
    assert_prints!(input, "hello");
}

#[test]
fn test_if_elseif_block_true_false() {
    let input = "
    IF 1 < 2 THEN
        PRINT \"hello\"
    ELSEIF 2 < 1 THEN
        PRINT \"bye\"
    END IF
    ";
    assert_prints!(input, "hello");
}

#[test]
fn test_if_elseif_block_false_true() {
    let input = "
    IF 2 < 1 THEN
        PRINT \"hello\"
    ELSEIF 1 < 2 THEN
        PRINT \"bye\"
    END IF
    ";
    assert_prints!(input, "bye");
}

#[test]
fn test_if_elseif_block_false_false() {
    let input = "
    IF 2 < 1 THEN
        PRINT \"hello\"
    ELSEIF 2 < 1 THEN
        PRINT \"bye\"
    END IF
    ";
    assert_prints_nothing!(input);
}

#[test]
fn test_if_elseif_else_block_true_true() {
    let input = "
    IF 1 < 2 THEN
        PRINT \"hello\"
    ELSEIF 1 < 2 THEN
        PRINT \"bye\"
    ELSE
        PRINT \"else\"
    END IF
    ";
    assert_prints!(input, "hello");
}

#[test]
fn test_if_elseif_else_block_true_false() {
    let input = "
    IF 1 < 2 THEN
        PRINT \"hello\"
    ELSEIF 2 < 1 THEN
        PRINT \"bye\"
    ELSE
        PRINT \"else\"
    END IF
    ";
    assert_prints!(input, "hello");
}

#[test]
fn test_if_elseif_else_block_false_true() {
    let input = "
    IF 2 < 1 THEN
        PRINT \"hello\"
    ELSEIF 1 < 2 THEN
        PRINT \"bye\"
    ELSE
        PRINT \"else\"
    END IF
    ";
    assert_prints!(input, "bye");
}

#[test]
fn test_if_elseif_else_block_false_false() {
    let input = "
    IF 2 < 1 THEN
        PRINT \"hello\"
    ELSEIF 2 < 1 THEN
        PRINT \"bye\"
    ELSE
        PRINT \"else\"
    END IF
    ";
    assert_prints!(input, "else");
}

#[test]
fn test_if_multiple_elseif_block() {
    let input = "
    IF 2 < 1 THEN
        PRINT \"hello\"
    ELSEIF 1 < 2 THEN
        PRINT \"bye\"
    ELSEIF 1 < 2 THEN
        PRINT \"else if 2\"
    END IF
    ";
    assert_prints!(input, "bye");
}

#[test]
fn test_single_line_if() {
    let input = r#"
    IF 1 THEN PRINT "hello"
    "#;
    assert_prints!(input, "hello");
    let input = r#"
    IF 0 THEN PRINT "hello"
    "#;
    assert_prints_nothing!(input);
    let input = r#"
    PRINT "before"
    IF 1 THEN PRINT "hello"
    PRINT "after"
    "#;
    assert_prints!(input, "before", "hello", "after");
    let input = r#"
    PRINT "before"
    IF 0 THEN PRINT "hello"
    PRINT "after"
    "#;
    assert_prints!(input, "before", "after");
}
