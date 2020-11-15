use crate::assert_linter_err;
use crate::common::{AtRowCol, QError};
use crate::linter::test_utils::linter_ok;
use crate::parser::{
    ArrayDimension, BareName, BuiltInStyle, DimName, DimType, Expression, ExpressionType,
    ParamName, ParamType, Statement, SubImplementation, TopLevelToken, TypeQualifier,
};

#[test]
fn test_passing_array_parameter_without_parenthesis() {
    let input = r#"
    DIM choice$(1 TO 3)

    Menu choice$

    SUB Menu(choice$())
    END SUB
    "#;

    assert_linter_err!(input, QError::ArgumentTypeMismatch, 4, 10);
}

#[test]
fn test_dim_array() {
    let input = r#"
    DIM choice$(1 TO 3)
    "#;

    assert_eq!(
        linter_ok(input),
        vec![TopLevelToken::Statement(Statement::Dim(
            DimName::new(
                "choice".into(),
                DimType::Array(
                    vec![ArrayDimension {
                        lbound: Some(Expression::IntegerLiteral(1).at_rc(2, 17)),
                        ubound: Expression::IntegerLiteral(3).at_rc(2, 22)
                    }],
                    Box::new(DimType::BuiltIn(
                        TypeQualifier::DollarString,
                        BuiltInStyle::Compact
                    ))
                )
            )
            .at_rc(2, 9)
        ))
        .at_rc(2, 5),]
    );
}

#[test]
fn test_sub_with_array_parameter() {
    let input = r#"
    SUB Menu(choice$())
    END SUB
    "#;

    assert_eq!(
        linter_ok(input),
        vec![TopLevelToken::SubImplementation(SubImplementation {
            name: BareName::from("Menu").at_rc(2, 9),
            params: vec![ParamName::new(
                "choice".into(),
                ParamType::Array(Box::new(ParamType::BuiltIn(
                    TypeQualifier::DollarString,
                    BuiltInStyle::Compact
                )))
            )
            .at_rc(2, 14)],
            body: vec![]
        })
        .at_rc(2, 5)]
    );
}

#[test]
fn test_passing_array_parameter_with_parenthesis() {
    let input = r#"
    DIM choice$(1 TO 3)

    Menu choice$()

    SUB Menu(choice$())
    END SUB
    "#;

    assert_eq!(
        linter_ok(input),
        vec![
            TopLevelToken::Statement(Statement::Dim(
                DimName::new(
                    "choice".into(),
                    DimType::Array(
                        vec![ArrayDimension {
                            lbound: Some(Expression::IntegerLiteral(1).at_rc(2, 17)),
                            ubound: Expression::IntegerLiteral(3).at_rc(2, 22)
                        }],
                        Box::new(DimType::BuiltIn(
                            TypeQualifier::DollarString,
                            BuiltInStyle::Compact
                        ))
                    )
                )
                .at_rc(2, 9)
            ))
            .at_rc(2, 5),
            TopLevelToken::Statement(Statement::SubCall(
                "Menu".into(),
                vec![Expression::Variable(
                    "choice$".into(),
                    ExpressionType::Array(Box::new(ExpressionType::BuiltIn(
                        TypeQualifier::DollarString
                    )))
                )
                .at_rc(4, 10)]
            ))
            .at_rc(4, 5),
            TopLevelToken::SubImplementation(SubImplementation {
                name: BareName::from("Menu").at_rc(6, 9),
                params: vec![ParamName::new(
                    "choice".into(),
                    ParamType::Array(Box::new(ParamType::BuiltIn(
                        TypeQualifier::DollarString,
                        BuiltInStyle::Compact
                    )))
                )
                .at_rc(6, 14)],
                body: vec![]
            })
            .at_rc(6, 5)
        ]
    );
}
