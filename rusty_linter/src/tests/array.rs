use crate::assert_linter_err;
use crate::core::LintError;
use crate::tests::test_utils::linter_ok;
use rusty_common::*;
use rusty_parser::BuiltInFunction;
use rusty_parser::*;

#[test]
fn test_passing_array_parameter_without_parenthesis() {
    let input = r#"
    DIM choice$(1 TO 3)

    Menu choice$

    SUB Menu(choice$())
    END SUB
    "#;

    assert_linter_err!(input, LintError::ArgumentTypeMismatch, 4, 10);
}

#[test]
fn test_dim_array() {
    let input = r#"
    DIM choice$(1 TO 3)
    "#;

    assert_eq!(
        linter_ok(input),
        vec![GlobalStatement::Statement(Statement::Dim(
            DimVar::new(
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
        vec![GlobalStatement::SubImplementation(SubImplementation {
            name: BareName::from("Menu").at_rc(2, 9),
            params: vec![Parameter::new(
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
            GlobalStatement::Statement(Statement::Dim(
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
            GlobalStatement::Statement(Statement::SubCall(
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
            GlobalStatement::SubImplementation(SubImplementation {
                name: BareName::from("Menu").at_rc(6, 9),
                params: vec![Parameter::new(
                    "choice".into(),
                    ParamType::Array(Box::new(ParamType::BuiltIn(
                        TypeQualifier::DollarString,
                        BuiltInStyle::Compact
                    )))
                )
                .at_rc(6, 14)],
                body: vec![
                    Statement::Dim(
                        DimVar::new_compact_local("X", TypeQualifier::DollarString)
                            .into_list_rc(7, 9)
                    )
                    .at_rc(7, 9),
                    Statement::assignment(
                        Expression::var_resolved("X$"),
                        Expression::ArrayElement(
                            "choice$".into(),
                            vec![Expression::IntegerLiteral(1).at_rc(7, 22)],
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
            GlobalStatement::Statement(Statement::Dim(
                DimVar::new_compact_local("X", TypeQualifier::BangSingle).into_list_rc(3, 5)
            ))
            .at_rc(3, 5),
            GlobalStatement::Statement(Statement::Dim(
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
                    .build()
                    .into_list_rc(2, 9)
            ))
            .at_rc(2, 5),
            GlobalStatement::Statement(Statement::assignment(
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
