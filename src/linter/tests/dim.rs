use crate::assert_linter_err;
use crate::common::QError;

#[test]
fn test_dim_duplicate_definition_same_builtin_type() {
    let program = r#"
            DIM A AS STRING
            DIM A AS STRING
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_duplicate_definition_different_builtin_type() {
    let program = r#"
            DIM A AS STRING
            DIM A AS INTEGER
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_after_const_duplicate_definition() {
    let program = r#"
            CONST A = "hello"
            DIM A AS STRING
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_after_variable_assignment_duplicate_definition() {
    let program = r#"
            A = 42
            DIM A AS INTEGER
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_string_duplicate_definition() {
    let program = r#"
            DIM A$
            DIM A$
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_bare_duplicate_definition() {
    let program = r#"
            DIM A
            DIM A
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_single_bare_duplicate_definition() {
    // single is the default type
    let program = r#"
            DIM A!
            DIM A
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_bare_single_duplicate_definition() {
    // single is the default type
    let program = r#"
            DIM A
            DIM A!
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_compact_bare_integer_duplicate_definition() {
    let program = r#"
            DEFINT A-Z
            DIM A
            DIM A%
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 4, 17);
}

#[test]
fn test_dim_extended_inside_sub_name_clashing_sub_name() {
    let program = r#"
            SUB Hello
            Dim Hello AS STRING
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_bare_inside_sub_name_clashing_other_sub_name() {
    let program = r#"
            SUB Oops
            END SUB

            SUB Hello
            Dim Oops
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 6, 17);
}

#[test]
fn test_dim_extended_inside_sub_name_clashing_param_name() {
    let program = r#"
            SUB Hello(Oops)
            Dim Oops AS STRING
            END SUB
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_extended_inside_function_name_clashing_function_name() {
    let program = r#"
            FUNCTION Hello
            Dim Hello AS STRING
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}

#[test]
fn test_dim_extended_inside_function_name_clashing_other_function_name() {
    let program = r#"
            FUNCTION Hello
            Dim Bar AS STRING
            END FUNCTION
            FUNCTION Bar
            END Function
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 17);
}
