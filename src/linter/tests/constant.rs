use crate::assert_linter_err;
use crate::assert_linter_ok_top_level_statements;
use crate::common::{AtRowCol, QError};
use crate::linter::test_utils::linter_ok;
use crate::linter::{
    Expression, ParamName, ParamType, PrintNode, Statement, SubImplementation, TopLevelToken,
};
use crate::parser::{BareName, BuiltInStyle, TypeQualifier};

#[test]
fn function_call_not_allowed() {
    let program = r#"
            CONST X = Add(1, 2)
            "#;
    assert_linter_err!(program, QError::InvalidConstant, 2, 23);
}

#[test]
fn variable_not_allowed() {
    let program = r#"
            X = 42
            CONST A = X + 1
            "#;
    assert_linter_err!(program, QError::InvalidConstant, 3, 23);
}

#[test]
fn variable_already_exists() {
    let program = "
            X = 42
            CONST X = 32
            ";
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
}

#[test]
fn variable_already_exists_as_sub_call_param() {
    let program = "
            INPUT X%
            CONST X = 1
            ";
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
}

#[test]
fn const_already_exists() {
    let program = "
            CONST X = 32
            CONST X = 33
            ";
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
}

#[test]
fn qualified_usage_from_string_literal() {
    let program = r#"
            CONST X! = "hello"
            "#;
    assert_linter_err!(program, QError::TypeMismatch, 2, 24);
}

#[test]
fn const_after_dim_duplicate_definition() {
    let program = r#"
            DIM A AS STRING
            CONST A = "hello"
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 3, 19);
}

#[test]
fn test_global_const_cannot_have_function_name() {
    let program = r#"
            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            CONST GetAction = 42
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 5, 19);
}

#[test]
fn test_local_const_cannot_have_function_name() {
    let program = r#"
            FUNCTION GetAction$
                GetAction$ = "hello"
            END FUNCTION
            FUNCTION Echo(X)
                CONST GetAction = 42
            END FUNCTION
            "#;
    assert_linter_err!(program, QError::DuplicateDefinition, 6, 23);
}

#[test]
fn test_forward_const_not_allowed() {
    let input = "
            CONST A = B + 1
            CONST B = 42";
    assert_linter_err!(input, QError::InvalidConstant, 2, 23);
}

#[test]
fn test_constant_definition_and_usage_in_print() {
    let program = r#"
    CONST X = "hello"
    PRINT X
    "#;
    assert_linter_ok_top_level_statements!(
        program,
        Statement::Print(PrintNode::one(
            Expression::StringLiteral("hello".to_owned()).at_rc(3, 11)
        ))
    );
}

#[test]
fn test_constant_definition_and_usage_in_sub_call_arg() {
    let program = r#"
    CONST X = "hello"
    MySub X

    SUB MySub(A$)
    END SUB
    "#;
    assert_eq!(
        linter_ok(program),
        vec![
            TopLevelToken::Statement(Statement::SubCall(
                "MySub".into(),
                vec![Expression::StringLiteral("hello".to_owned()).at_rc(3, 11)]
            ))
            .at_rc(3, 5),
            TopLevelToken::SubImplementation(SubImplementation {
                name: BareName::from("MySub").at_rc(5, 9),
                params: vec![ParamName::new(
                    "A".into(),
                    ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                )
                .at_rc(5, 15)],
                body: vec![]
            })
            .at_rc(5, 5)
        ]
    );
}
