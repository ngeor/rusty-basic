use crate::assert_linter_err;
use crate::assert_linter_ok_top_level_statements;
use crate::common::*;
use crate::linter::test_utils::*;
use crate::linter::*;
use crate::parser::TypeQualifier;
use std::collections::HashMap;

/// Three step tests:
/// 1. DIM a new variable
/// 2. Assign to the variable
/// 3. Use it in an expression

#[test]
fn bare() {
    let program = r#"
    DIM A
    A = 42
    PRINT A
    "#;
    assert_eq!(
        linter_ok(program),
        vec![
            TopLevelToken::Statement(Statement::Dim(DimName::parse("A!").at_rc(2, 9))).at_rc(2, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::var("A!"),
                Expression::IntegerLiteral(42).at_rc(3, 9)
            ))
            .at_rc(3, 5),
            TopLevelToken::Statement(Statement::Print(PrintNode::one(
                Expression::var("A!").at_rc(4, 11)
            )))
            .at_rc(4, 5)
        ]
    );
}

#[test]
fn compact_string() {
    let program = r#"
    DIM A$
    A$ = "hello"
    PRINT A$
    "#;
    assert_eq!(
        linter_ok(program),
        vec![
            TopLevelToken::Statement(Statement::Dim(DimName::parse("A$").at_rc(2, 9))).at_rc(2, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::var("A$"),
                Expression::StringLiteral("hello".to_string()).at_rc(3, 10)
            ))
            .at_rc(3, 5),
            TopLevelToken::Statement(Statement::Print(PrintNode::one(
                Expression::var("A$").at_rc(4, 11)
            )))
            .at_rc(4, 5)
        ]
    );
}

#[test]
fn extended_string() {
    let program = r#"
    DIM A AS STRING
    A = "hello"
    PRINT A
    "#;
    assert_eq!(
        linter_ok(program),
        vec![
            TopLevelToken::Statement(Statement::Dim(DimName::parse("A$").at_rc(2, 9))).at_rc(2, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::var("A$"),
                Expression::StringLiteral("hello".to_string()).at_rc(3, 9)
            ))
            .at_rc(3, 5),
            TopLevelToken::Statement(Statement::Print(PrintNode::one(
                Expression::var("A$").at_rc(4, 11)
            )))
            .at_rc(4, 5)
        ]
    );
}

#[test]
fn user_defined_type() {
    let input = r#"
    TYPE Card
        Value AS INTEGER
        Suit AS STRING * 9
    END TYPE
    DIM A AS Card
    DIM B AS Card
    A = B
    "#;
    let (program, user_defined_types) = linter_ok_with_types(input);
    assert_eq!(
        program,
        vec![
            TopLevelToken::Statement(Statement::Dim(
                DimName::user_defined("A", "Card").at_rc(6, 9)
            ))
            .at_rc(6, 5),
            TopLevelToken::Statement(Statement::Dim(
                DimName::user_defined("B", "Card").at_rc(7, 9)
            ))
            .at_rc(7, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::user_defined("A", "Card"),
                Expression::user_defined("B", "Card").at_rc(8, 9)
            ))
            .at_rc(8, 5)
        ]
    );
    assert_eq!(
        user_defined_types.len(),
        1,
        "Expected one user defined type"
    );
    assert!(
        user_defined_types.contains_key(&"Card".into()),
        "Expected to contain the `Card` type"
    );
    let mut m: HashMap<CaseInsensitiveString, ElementType> = HashMap::new();
    m.insert("Value".into(), ElementType::Integer);
    m.insert("Suit".into(), ElementType::FixedLengthString(9));
    assert_eq!(
        *user_defined_types.get(&"Card".into()).unwrap(),
        UserDefinedType::new(m)
    );
}

#[test]
fn user_defined_type_integer_element() {
    let input = r#"
    TYPE Card
        Value AS INTEGER
        Suit AS STRING * 9
    END TYPE
    DIM A AS Card
    A.Value = 42
    PRINT A.Value
    "#;
    assert_linter_ok_top_level_statements!(
        input,
        Statement::Dim(DimName::user_defined("A", "Card").at_rc(6, 9)),
        Statement::Assignment(
            Expression::Property(
                Box::new(Expression::user_defined("A", "Card")),
                "Value".into(),
                ExpressionType::BuiltIn(TypeQualifier::PercentInteger)
            ),
            Expression::IntegerLiteral(42).at_rc(7, 15)
        ),
        Statement::Print(PrintNode::one(
            Expression::Property(
                Box::new(Expression::user_defined("A", "Card")),
                "Value".into(),
                ExpressionType::BuiltIn(TypeQualifier::PercentInteger)
            )
            .at_rc(8, 11)
        ))
    );
}

#[test]
fn user_defined_type_string_element() {
    let input = r#"
    TYPE Card
        Value AS INTEGER
        Suit AS STRING * 9
    END TYPE
    DIM A AS Card
    A.Suit = "diamonds"
    PRINT A.Suit
    "#;
    assert_linter_ok_top_level_statements!(
        input,
        Statement::Dim(DimName::user_defined("A", "Card").at_rc(6, 9)),
        Statement::Assignment(
            Expression::Property(
                Box::new(Expression::user_defined("A", "Card")),
                "Suit".into(),
                ExpressionType::FixedLengthString(9)
            ),
            Expression::StringLiteral("diamonds".to_owned()).at_rc(7, 14)
        ),
        Statement::Print(PrintNode::one(
            Expression::Property(
                Box::new(Expression::user_defined("A", "Card")),
                "Suit".into(),
                ExpressionType::FixedLengthString(9)
            )
            .at_rc(8, 11)
        ))
    );
}

#[test]
fn element_type_qualified_wrong_type() {
    let program = r#"
    TYPE Card
        Value AS INTEGER
    END TYPE
    DIM c AS Card
    c.Value! = 3
    "#;
    assert_linter_err!(program, QError::TypeMismatch, 6, 5);
}
