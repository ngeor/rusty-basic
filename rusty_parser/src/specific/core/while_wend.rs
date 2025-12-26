use crate::error::ParseError;
use crate::pc::*;
use crate::specific::core::expression::ws_expr_pos_p;
use crate::specific::core::statement::ConditionalBlock;
use crate::specific::core::statements::ZeroOrMoreStatements;
use crate::specific::pc_specific::*;
use crate::specific::*;

pub fn while_wend_p() -> impl Parser<RcStringView, Output = Statement> {
    seq4(
        keyword(Keyword::While),
        ws_expr_pos_p().or_syntax_error("Expected: expression after WHILE"),
        ZeroOrMoreStatements::new_with_custom_error(Keyword::Wend, ParseError::WhileWithoutWend),
        keyword(Keyword::Wend).or_fail(ParseError::WhileWithoutWend),
        |_, condition, statements, _| {
            Statement::While(ConditionalBlock {
                condition,
                statements,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::error::ParseError;
    use crate::specific::*;
    use crate::test_utils::*;
    use crate::*;
    use rusty_common::*;
    #[test]
    fn test_while_wend_leading_whitespace() {
        let input = "
        WHILE A < 5
            SYSTEM
        WEND
        ";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::While(ConditionalBlock {
                condition: Expression::BinaryExpression(
                    Operator::Less,
                    Box::new("A".as_var_expr(2, 15)),
                    Box::new(5.as_lit_expr(2, 19)),
                    ExpressionType::Unresolved
                )
                .at_rc(2, 17),
                statements: vec![Statement::System.at_rc(3, 13)]
            })
        );
    }

    #[test]
    fn test_while_wend_single_line() {
        let program = parse("WHILE A < 5: A = A + 1: Flint A: WEND").demand_single_statement();
        assert_eq!(
            program,
            Statement::While(ConditionalBlock {
                condition: Expression::BinaryExpression(
                    Operator::Less,
                    Box::new("A".as_var_expr(1, 7)),
                    Box::new(5.as_lit_expr(1, 11)),
                    ExpressionType::Unresolved
                )
                .at_rc(1, 9),
                statements: vec![
                    Statement::assignment(
                        Expression::var_unresolved("A"),
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("A".as_var_expr(1, 18)),
                            Box::new(1.as_lit_expr(1, 22)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(1, 20)
                    )
                    .at_rc(1, 14),
                    Statement::SubCall(BareName::from("Flint"), vec!["A".as_var_expr(1, 31)])
                        .at_rc(1, 25)
                ]
            })
        );
    }

    #[test]
    fn test_inline_comment() {
        let input = "
        WHILE A    ' keep looping
            SYSTEM ' exit
        WEND       ' end of loop
        ";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::While(ConditionalBlock {
                    condition: "A".as_var_expr(2, 15),
                    statements: vec![
                        Statement::Comment(" keep looping".to_string()).at_rc(2, 20),
                        Statement::System.at_rc(3, 13),
                        Statement::Comment(" exit".to_string()).at_rc(3, 20)
                    ]
                }))
                .at_rc(2, 9),
                GlobalStatement::Statement(Statement::Comment(" end of loop".to_string()))
                    .at_rc(4, 20)
            ]
        );
    }

    #[test]
    fn test_wend_without_while() {
        let input = "WEND";
        assert_parser_err!(input, ParseError::WendWithoutWhile);
    }

    #[test]
    fn test_while_without_wend() {
        let input = r#"
        WHILE X > 0
        PRINT X
        "#;
        assert_parser_err!(input, ParseError::WhileWithoutWend);
    }

    #[test]
    fn test_while_condition_in_parenthesis() {
        let input = r#"
        WHILE(X > 0)
            Flint X
        WEND"#;
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::While(ConditionalBlock {
                condition: Expression::Parenthesis(Box::new(
                    Expression::BinaryExpression(
                        Operator::Greater,
                        Box::new("X".as_var_expr(2, 15)),
                        Box::new(0.as_lit_expr(2, 19)),
                        ExpressionType::Unresolved
                    )
                    .at_rc(2, 17)
                ))
                .at_rc(2, 14),
                statements: vec![
                    Statement::SubCall("Flint".into(), vec!["X".as_var_expr(3, 19)]).at_rc(3, 13)
                ]
            })
        );
    }

    #[test]
    fn test_while_wend_without_colon_separator() {
        let input = r#"
        WHILE X > 0
            PRINT X WEND
        "#;
        assert_parser_err!(
            input,
            ParseError::syntax_error("Expected: statement separator"),
            3,
            20
        );
    }

    #[test]
    fn test_while_wend_wend_same_line_on_last_statement() {
        let program = parse(
            r#"
        WHILE A < 5
            A = A + 1
            Flint A: WEND"#,
        )
        .demand_single_statement();
        assert_eq!(
            program,
            Statement::While(ConditionalBlock {
                condition: Expression::BinaryExpression(
                    Operator::Less,
                    Box::new("A".as_var_expr(2, 15)),
                    Box::new(5.as_lit_expr(2, 19)),
                    ExpressionType::Unresolved
                )
                .at_rc(2, 17),
                statements: vec![
                    Statement::assignment(
                        Expression::var_unresolved("A"),
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("A".as_var_expr(3, 17)),
                            Box::new(1.as_lit_expr(3, 21)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(3, 19)
                    )
                    .at_rc(3, 13),
                    Statement::SubCall(BareName::from("Flint"), vec!["A".as_var_expr(4, 19)])
                        .at_rc(4, 13)
                ]
            })
        );
    }
}
