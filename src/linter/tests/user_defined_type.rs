use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn duplicate_type_throws_duplicate_definition() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            TYPE Card
                Value AS INTEGER
            END TYPE";
    assert_linter_err!(input, QError::DuplicateDefinition, 6, 13);
}

#[test]
fn duplicate_element_name() {
    let input = "
            TYPE Card
                Value AS INTEGER
                Value AS INTEGER
            END TYPE
            ";
    assert_linter_err!(input, QError::DuplicateDefinition, 4, 17);
}

#[test]
fn element_using_container_type_throws_type_not_defined() {
    let input = "
            TYPE Card
                Item AS Card
            END TYPE";
    // TODO QBasic actually positions the error on the "AS" keyword
    assert_linter_err!(input, QError::TypeNotDefined, 3, 25);
}

#[test]
fn dim_using_undefined_type() {
    let input = "DIM X AS Card";
    // TODO QBasic actually positions the error on the "AS" keyword
    assert_linter_err!(input, QError::TypeNotDefined, 1, 5);
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
    assert_linter_err!(input, QError::TypeNotDefined, 3, 29);
}

#[test]
fn string_length_must_be_constant() {
    let input = "
            TYPE Invalid
                N AS STRING * A
            END TYPE";
    assert_linter_err!(input, QError::InvalidConstant, 3, 31);
}

#[test]
fn string_length_must_be_constant_const_cannot_follow_type() {
    let input = "
            TYPE Invalid
                N AS STRING * A
            END TYPE

            CONST A = 10";
    assert_linter_err!(input, QError::InvalidConstant, 3, 31);
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
    assert_linter_err!(input, QError::ElementNotDefined, 8, 19);
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
    assert_linter_err!(input, QError::DotClash, 8, 17);
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
    assert_linter_err!(input, QError::DotClash, 7, 17);
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
        // QBasic uses the right side expr for the location
        assert_linter_err!(input, QError::TypeMismatch, 9, 26 + (op.len() as u32) + 1);
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
        assert_linter_err!(input, QError::TypeMismatch, 9, 25);
    }
}
