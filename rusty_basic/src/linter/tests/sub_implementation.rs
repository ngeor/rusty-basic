use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn test_sub_param_clashing_sub_name() {
    let program = r#"
            SUB Hello(Hello)
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 2, 23);
}

#[test]
fn test_sub_param_clashing_other_sub_name_declared_earlier() {
    let program = r#"
            SUB Hello
            END SUB
            SUB Goodbye(Hello)
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 4, 25);
}

#[test]
fn test_sub_param_clashing_other_sub_name_declared_later() {
    let program = r#"
            SUB Goodbye(Hello)
            END SUB
            SUB Hello
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 2, 25);
}

#[test]
fn test_sub_param_duplicate() {
    let program = r#"
            SUB Hello(A, A)
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 2, 26);
}

#[test]
fn test_sub_param_extended_duplicate() {
    let program = r#"
            SUB Hello(A AS INTEGER, A AS STRING)
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 2, 37);
}

#[test]
fn test_cannot_override_built_in_sub_with_declaration() {
    let program = r#"
            DECLARE SUB Environ
            PRINT "Hello"
            SUB Environ
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 4, 13);
}

#[test]
fn test_cannot_override_built_in_sub_without_declaration() {
    let program = r#"
            PRINT "Hello"
            SUB Environ
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 13);
}

#[test]
fn test_by_ref_parameter_type_mismatch() {
    let program = "
            DECLARE SUB Hello(N)
            A% = 42
            Hello A%
            SUB Hello(N)
                N = N + 1
            END SUB
            ";
    assert_linter_err!(program, QError::ArgumentTypeMismatch, 4, 19);
}

#[test]
fn test_by_ref_parameter_type_mismatch_user_defined_type() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            DIM c AS Card
            Test c.Value

            SUB Test(N)
            END SUB
            ";
    assert_linter_err!(input, QError::ArgumentTypeMismatch, 7, 18);
}

#[test]
fn test_sub_dotted_name_clashes_variable_of_user_defined_type() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            DIM A AS Card

            SUB A.B
            END SUB
            ";
    // QBasic actually reports the error on the dot
    assert_linter_err!(input, QError::DotClash, 8, 17);
}

#[test]
fn test_sub_dotted_name_clashes_variable_of_user_defined_type_in_other_sub() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B
            END SUB

            SUB C.D
                DIM A AS Card
            END SUB
            ";
    assert_linter_err!(input, QError::DotClash, 6, 17);
}

#[test]
fn test_sub_dotted_name_clashes_variable_of_user_defined_type_in_other_sub_following() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B
                DIM C AS Card
            END SUB

            SUB C.D
            END SUB
            ";
    assert_linter_err!(input, QError::DotClash, 10, 17);
}

#[test]
fn test_sub_param_dotted_name_clashes_variable_of_user_defined_type_in_other_sub_following() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B
                DIM C AS Card
            END SUB

            SUB Oops(C.D AS INTEGER)
            END SUB
            ";
    assert_linter_err!(input, QError::DotClash, 10, 22);
}

#[test]
fn test_sub_param_dotted_name_clashes_param_of_user_defined_type_in_other_sub_following() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B(C AS Card)
            END SUB

            SUB Oops(C.D AS INTEGER)
            END SUB
            ";
    assert_linter_err!(input, QError::DotClash, 9, 22);
}

#[test]
fn test_sub_variable_dotted_name_clashes_variable_of_user_defined_type_in_other_sub() {
    let input = "
            TYPE Card
                Value AS INTEGER
            END TYPE

            SUB A.B
                DIM X AS Card
            END SUB

            SUB Oops
                DIM X.Y AS INTEGER
            END SUB
            ";
    assert_linter_err!(input, QError::DotClash, 11, 21);
}
