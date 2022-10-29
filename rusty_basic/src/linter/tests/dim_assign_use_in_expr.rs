use crate::assert_linter_err;
use crate::assert_linter_ok_top_level_statements;
use crate::linter::test_utils::*;
use crate::linter::HasUserDefinedTypes;
use crate::parser::{
    BareName, BuiltInStyle, DimName, DimType, Element, ElementType, Expression, ExpressionType,
    PrintNode, Statement, TopLevelToken, TypeQualifier,
};
use rusty_common::*;

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
            TopLevelToken::Statement(Statement::Dim(DimName::parse("A!").into_list_rc(2, 9)))
                .at_rc(2, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::var_resolved("A!"),
                Expression::IntegerLiteral(42).at_rc(3, 9)
            ))
            .at_rc(3, 5),
            TopLevelToken::Statement(Statement::Print(PrintNode::one(
                Expression::var_resolved("A!").at_rc(4, 11)
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
            TopLevelToken::Statement(Statement::Dim(DimName::parse("A$").into_list_rc(2, 9)))
                .at_rc(2, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::var_resolved("A$"),
                Expression::StringLiteral("hello".to_string()).at_rc(3, 10)
            ))
            .at_rc(3, 5),
            TopLevelToken::Statement(Statement::Print(PrintNode::one(
                Expression::var_resolved("A$").at_rc(4, 11)
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
            TopLevelToken::Statement(Statement::Dim(
                DimName::new(
                    "A".into(),
                    DimType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Extended),
                )
                .into_list_rc(2, 9)
            ))
            .at_rc(2, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::var_resolved("A$"),
                Expression::StringLiteral("hello".to_string()).at_rc(3, 9)
            ))
            .at_rc(3, 5),
            TopLevelToken::Statement(Statement::Print(PrintNode::one(
                Expression::var_resolved("A$").at_rc(4, 11)
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
    let (program, user_defined_types_holder) = linter_ok_with_types(input);
    assert_eq!(
        program,
        vec![
            TopLevelToken::Statement(Statement::Dim(
                DimName::new(
                    "A".into(),
                    DimType::UserDefined(BareName::from("Card").at_rc(6, 14)),
                )
                .into_list_rc(6, 9)
            ))
            .at_rc(6, 5),
            TopLevelToken::Statement(Statement::Dim(
                DimName::new(
                    "B".into(),
                    DimType::UserDefined(BareName::from("Card").at_rc(7, 14)),
                )
                .into_list_rc(7, 9)
            ))
            .at_rc(7, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::var_user_defined("A", "Card"),
                Expression::var_user_defined("B", "Card").at_rc(8, 9)
            ))
            .at_rc(8, 5)
        ]
    );
    let user_defined_types = user_defined_types_holder.user_defined_types();
    assert_eq!(
        user_defined_types.len(),
        1,
        "Expected one user defined type"
    );
    let opt_card_type = user_defined_types.get(&"Card".into()).cloned();
    assert!(
        opt_card_type.is_some(),
        "Expected to contain the `Card` type"
    );
    let card_type = opt_card_type.unwrap();
    assert_eq!(card_type.bare_name(), &BareName::from("Card"));
    let elements: Vec<Element> = card_type.elements().map(|x| x.as_ref().clone()).collect();
    assert_eq!(
        elements,
        vec![
            Element::new("Value".into(), ElementType::Integer, vec![]),
            Element::new(
                "Suit".into(),
                ElementType::FixedLengthString(Expression::IntegerLiteral(9).at_rc(4, 26), 9),
                vec![]
            )
        ]
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
        Statement::Dim(
            DimName::new(
                "A".into(),
                DimType::UserDefined(BareName::from("Card").at_rc(6, 14)),
            )
            .into_list_rc(6, 9)
        ),
        Statement::Assignment(
            Expression::Property(
                Box::new(Expression::var_user_defined("A", "Card")),
                "Value".into(),
                ExpressionType::BuiltIn(TypeQualifier::PercentInteger)
            ),
            Expression::IntegerLiteral(42).at_rc(7, 15)
        ),
        Statement::Print(PrintNode::one(
            Expression::Property(
                Box::new(Expression::var_user_defined("A", "Card")),
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
        Statement::Dim(
            DimName::new(
                "A".into(),
                DimType::UserDefined(BareName::from("Card").at_rc(6, 14)),
            )
            .into_list_rc(6, 9)
        ),
        Statement::Assignment(
            Expression::Property(
                Box::new(Expression::var_user_defined("A", "Card")),
                "Suit".into(),
                ExpressionType::FixedLengthString(9)
            ),
            Expression::StringLiteral("diamonds".to_owned()).at_rc(7, 14)
        ),
        Statement::Print(PrintNode::one(
            Expression::Property(
                Box::new(Expression::var_user_defined("A", "Card")),
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
