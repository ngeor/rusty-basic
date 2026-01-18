use rusty_pc::*;

use crate::core::expression::ws_expr_pos_p;
use crate::core::statement::ConditionalBlock;
use crate::core::statements::zero_or_more_statements;
use crate::error::ParserError;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::*;

pub fn while_wend_p() -> impl Parser<StringView, Output = Statement, Error = ParserError> {
    seq4(
        keyword(Keyword::While),
        ws_expr_pos_p().or_expected("expression after WHILE"),
        zero_or_more_statements!(Keyword::Wend, ParserErrorKind::WhileWithoutWend),
        keyword(Keyword::Wend).or_fail(ParserError::WhileWithoutWend),
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
    use rusty_common::*;

    use crate::test_utils::*;
    use crate::{assert_parser_err, *};
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
                    Statement::sub_call(BareName::from("Flint"), vec!["A".as_var_expr(1, 31)])
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
        assert_parser_err!(input, ParserError::WendWithoutWhile);
    }

    #[test]
    fn test_while_without_wend() {
        let input = r#"
        WHILE X > 0
        PRINT X
        "#;
        assert_parser_err!(input, ParserError::WhileWithoutWend);
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
                    Statement::sub_call("Flint".into(), vec!["X".as_var_expr(3, 19)]).at_rc(3, 13)
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
        assert_parser_err!(input, expected("end-of-statement"), 3, 20);
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
                    Statement::sub_call(BareName::from("Flint"), vec!["A".as_var_expr(4, 19)])
                        .at_rc(4, 13)
                ]
            })
        );
    }
}
