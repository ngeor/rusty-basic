use crate::assert_linter_err;
use crate::LintError;

#[test]
fn duplicate_type_throws_duplicate_definition() {
    let input = "
    TYPE Card
        Value AS INTEGER
    END TYPE

    TYPE Card
        Value AS INTEGER
    END TYPE";
    assert_linter_err!(input, LintError::DuplicateDefinition, 6, 5);
}

#[test]
fn duplicate_element_name() {
    let input = "
    TYPE Card
        Value AS INTEGER
        Value AS INTEGER
    END TYPE
    ";
    assert_linter_err!(input, LintError::DuplicateDefinition, 4, 9);
}

#[test]
fn element_using_container_type_throws_type_not_defined() {
    let input = "
    TYPE Card
        Item AS Card
    END TYPE";
    // TODO QBasic actually positions the error on the "AS" keyword
    assert_linter_err!(input, LintError::TypeNotDefined, 3, 17);
}

#[test]
fn dim_using_undefined_type() {
    let input = "DIM X AS Card";
    // TODO QBasic actually positions the error on the "AS" keyword
    assert_linter_err!(input, LintError::TypeNotDefined, 1, 10);
}

#[test]
fn using_type_before_defined_throws_type_not_defined() {
    let input = "
    TYPE Address
        PostCode AS PostCode
    END TYPE

    TYPE PostCode
        Prefix AS INTEGER
        Suffix AS STRING * 2
    END TYPE";
    assert_linter_err!(input, LintError::TypeNotDefined, 3, 21);
}

#[test]
fn string_length_must_be_constant() {
    let input = "
    TYPE Invalid
        N AS STRING * A
    END TYPE";
    assert_linter_err!(input, LintError::InvalidConstant, 3, 23);
}

#[test]
fn string_length_must_be_constant_const_cannot_follow_type() {
    let input = "
    TYPE Invalid
        N AS STRING * A
    END TYPE

    CONST A = 10";
    assert_linter_err!(input, LintError::InvalidConstant, 3, 23);
}

#[test]
fn string_length_illegal_expression() {
    let illegal_expressions = [
        "0",
        "-1",
        "3.14",
        "6.28#",
        "\"hello\"",
        "Foo(1)",
        "(1+1)",
        "32768", /* MAX_INT (32767) + 1*/
    ];
    for e in &illegal_expressions {
        let input = format!(
            "
            TYPE Invalid
                ZeroString AS STRING * {}
            END TYPE",
            e
        );
        assert_linter_err!(&input, LintError::InvalidConstant);
    }
}

#[test]
fn referencing_non_existing_member() {
    let input = "
    TYPE Card
        Suit AS STRING * 9
        Value AS INTEGER
    END TYPE

    DIM c AS Card
    PRINT c.Suite";
    // TODO QBasic reports the error at the dot
    assert_linter_err!(input, LintError::ElementNotDefined, 8, 11);
}

#[test]
fn referencing_existing_member_with_wrong_qualifier() {
    let input = "
    TYPE Card
        Suit AS STRING * 9
        Value AS INTEGER
    END TYPE

    DIM c AS Card
    PRINT c.Suit%";
    assert_linter_err!(input, LintError::TypeMismatch, 8, 11);
}

#[test]
fn cannot_define_variable_with_dot_if_clashes_with_user_defined_type() {
    let input = "
    TYPE Card
        Suit AS STRING * 9
        Value AS INTEGER
    END TYPE

    DIM C AS Card
    DIM C.Oops AS STRING
    ";
    // QBasic actually throws "Expected: , or end-of-statement" at the period position
    assert_linter_err!(input, LintError::DotClash, 8, 9);
}

#[test]
fn cannot_define_variable_with_dot_if_clashes_with_user_defined_type_reverse() {
    let input = "
    TYPE Card
        Suit AS STRING * 9
        Value AS INTEGER
    END TYPE

    DIM C.Oops AS STRING
    DIM C AS Card
    ";
    // QBasic actually throws "Expected: , or end-of-statement" at the period position
    assert_linter_err!(input, LintError::DotClash, 7, 9);
}

#[test]
fn cannot_use_in_binary_expression() {
    let ops = [
        "=", "<>", ">=", ">", "<", "<=", "+", "-", "*", "/", "AND", "OR",
    ];
    for op in &ops {
        let input = format!(
            "
            TYPE Card
                Value AS INTEGER
            END TYPE

            DIM a AS CARD
            DIM b AS CARD

            IF a {} b THEN
            END IF",
            op
        );
        // QBasic uses the right side expr for the position
        assert_linter_err!(
            &input,
            LintError::TypeMismatch,
            9,
            18 + (op.len() as u32) + 1
        );
    }
}

#[test]
fn cannot_use_in_unary_expression() {
    let ops = ["-", "NOT "];
    for op in &ops {
        let input = format!(
            "
            TYPE Card
                Value AS INTEGER
            END TYPE

            DIM a AS CARD
            DIM b AS CARD

            b = {}A",
            op
        );
        assert_linter_err!(&input, LintError::TypeMismatch);
    }
}
