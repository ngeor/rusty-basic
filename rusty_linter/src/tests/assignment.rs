use rusty_common::AtPos;
use rusty_parser::{DimVar, Expression, ExpressionType, Operator, Statement, TypeQualifier};

use crate::core::LintError;
use crate::{assert_linter_err, assert_linter_ok_global_statements};

#[test]
fn name_clashes_with_other_sub_name() {
    let program = r#"
            SUB Hello
            END SUB
            SUB Oops
            Hello = 2
            END SUB
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 5, 13);
}

#[test]
fn literals_type_mismatch() {
    assert_linter_err!("X = \"hello\"", LintError::TypeMismatch, 1, 5);
    assert_linter_err!("X! = \"hello\"", LintError::TypeMismatch, 1, 6);
    assert_linter_err!("X# = \"hello\"", LintError::TypeMismatch, 1, 6);
    assert_linter_err!("A$ = 1.0", LintError::TypeMismatch, 1, 6);
    assert_linter_err!("A$ = 1", LintError::TypeMismatch, 1, 6);
    assert_linter_err!("A$ = -1", LintError::TypeMismatch, 1, 6);
    assert_linter_err!("X% = \"hello\"", LintError::TypeMismatch, 1, 6);
    assert_linter_err!("X& = \"hello\"", LintError::TypeMismatch, 1, 6);
}

#[test]
fn assign_to_const() {
    let program = "
            CONST X = 3.14
            X = 6.28
            ";
    assert_linter_err!(program, LintError::DuplicateDefinition, 3, 13);
}

#[test]
fn assign_to_parent_const() {
    let program = r#"
            CONST X = 42
            SUB Hello
            X = 3
            END SUB
            "#;
    assert_linter_err!(program, LintError::DuplicateDefinition, 4, 13);
}

#[test]
fn assign_integer_to_extended_string() {
    let program = r#"
            X = 1
            IF X = 0 THEN DIM A AS STRING
            A = 42
            "#;
    assert_linter_err!(program, LintError::TypeMismatch, 4, 17);
}

#[test]
fn test_assign_binary_plus() {
    assert_linter_ok_global_statements!(
        "X% = 1 + 2.1",
        Statement::Dim(
            DimVar::new_compact_local("X", TypeQualifier::PercentInteger).into_list_rc(1, 1)
        ),
        Statement::assignment(
            Expression::var_resolved("X%"),
            Expression::BinaryExpression(
                Operator::Plus,
                Box::new(Expression::IntegerLiteral(1).at_rc(1, 6)),
                Box::new(Expression::SingleLiteral(2.1).at_rc(1, 10)),
                ExpressionType::BuiltIn(TypeQualifier::BangSingle),
            )
            .at_rc(1, 8)
        )
    );
}

#[test]
fn test_possible_property_folded_back_to_variable() {
    assert_linter_ok_global_statements!(
        "A.B = 12",
        Statement::Dim(
            DimVar::new_compact_local("A.B", TypeQualifier::BangSingle).into_list_rc(1, 1)
        ),
        Statement::assignment(
            Expression::var_resolved("A.B!"),
            Expression::IntegerLiteral(12).at_rc(1, 7),
        )
    );
}
