use crate::assert_linter_err;
use crate::linter::test_utils::linter_ok;
use rusty_common::*;
use rusty_parser::test_utils::ExpressionNodeLiteralFactory;
use rusty_parser::*;

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
                ),
            )
            .into_list_rc(2, 9)
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
            body: vec![],
            is_static: false
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
        X$ = choice$(1)
    END SUB
    "#;

    assert_eq!(
        linter_ok(input),
        vec![
            TopLevelToken::Statement(Statement::Dim(
                DimNameBuilder::new()
                    .bare_name("choice")
                    .dim_type(DimType::Array(
                        vec![ArrayDimension {
                            lbound: Some(Expression::IntegerLiteral(1).at_rc(2, 17)),
                            ubound: Expression::IntegerLiteral(3).at_rc(2, 22)
                        }],
                        Box::new(DimType::BuiltIn(
                            TypeQualifier::DollarString,
                            BuiltInStyle::Compact
                        ))
                    ))
                    .build_list_rc(2, 9)
            ))
            .at_rc(2, 5),
            TopLevelToken::Statement(Statement::SubCall(
                "Menu".into(),
                vec![Expression::ArrayElement(
                    "choice$".into(),
                    vec![],
                    VariableInfo {
                        expression_type: ExpressionType::BuiltIn(TypeQualifier::DollarString),
                        shared: false,
                        redim_info: None
                    }
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
                body: vec![
                    Statement::Dim(
                        DimName::new_compact_local("X", TypeQualifier::DollarString)
                            .into_list_rc(7, 9)
                    )
                    .at_rc(7, 9),
                    Statement::Assignment(
                        Expression::var_resolved("X$"),
                        Expression::ArrayElement(
                            "choice$".into(),
                            vec![1.as_lit_expr(7, 22)],
                            VariableInfo {
                                expression_type: ExpressionType::BuiltIn(
                                    TypeQualifier::DollarString
                                ),
                                shared: false,
                                redim_info: None
                            }
                        )
                        .at_rc(7, 14)
                    )
                    .at_rc(7, 9)
                ],
                is_static: false
            })
            .at_rc(6, 5)
        ]
    );
}

#[test]
fn test_passing_array_without_parenthesis() {
    let input = r#"
    DIM choice$(1 TO 3)
    X = LBound(choice$)
    "#;
    assert_eq!(
        linter_ok(input),
        vec![
            // X is hoisted first, even though it's implicitly defined later at line 3
            TopLevelToken::Statement(Statement::Dim(
                DimName::new_compact_local("X", TypeQualifier::BangSingle).into_list_rc(3, 5)
            ))
            .at_rc(3, 5),
            TopLevelToken::Statement(Statement::Dim(
                DimNameBuilder::new()
                    .bare_name("choice")
                    .dim_type(DimType::Array(
                        vec![ArrayDimension {
                            lbound: Some(1.as_lit_expr(2, 17)),
                            ubound: 3.as_lit_expr(2, 22)
                        }],
                        Box::new(DimType::BuiltIn(
                            TypeQualifier::DollarString,
                            BuiltInStyle::Compact
                        ))
                    ))
                    .build()
                    .into_list_rc(2, 9)
            ))
            .at_rc(2, 5),
            TopLevelToken::Statement(Statement::Assignment(
                Expression::Variable(
                    "X!".into(),
                    VariableInfo::new_local(ExpressionType::BuiltIn(TypeQualifier::BangSingle))
                ),
                Expression::BuiltInFunctionCall(
                    BuiltInFunction::LBound,
                    vec![Expression::Variable(
                        "choice$".into(),
                        VariableInfo::new_local(ExpressionType::Array(Box::new(
                            ExpressionType::BuiltIn(TypeQualifier::DollarString)
                        )))
                    )
                    .at_rc(3, 16)]
                )
                .at_rc(3, 9)
            ))
            .at_rc(3, 5)
        ]
    );
}
