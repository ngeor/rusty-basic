use crate::common::*;
use crate::parser::base::and_pc::AndDemandTrait;
use crate::parser::base::parsers::{FnMapTrait, Parser};
use crate::parser::expression::guarded_expression_node_p;
use crate::parser::specific::{keyword, MapErrTrait, OrSyntaxErrorTrait};
use crate::parser::statements::*;
use crate::parser::types::*;

pub fn while_wend_p() -> impl Parser<Output = Statement> {
    keyword(Keyword::While)
        .and_demand(guarded_expression_node_p().or_syntax_error("Expected: expression after WHILE"))
        .and_demand(
            zero_or_more_statements_opt_lazy(&[Keyword::Wend])
                .or_syntax_error("Expected statements"),
        )
        .and_demand(keyword(Keyword::Wend).map_err(QError::WhileWithoutWend))
        .fn_map(|(((_, condition), statements), _)| {
            Statement::While(ConditionalBlockNode {
                condition,
                statements,
            })
        })
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::{
        BareName, ConditionalBlockNode, Expression, ExpressionType, Operator, Statement,
        TopLevelToken,
    };

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
            Statement::While(ConditionalBlockNode {
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
            Statement::While(ConditionalBlockNode {
                condition: Expression::BinaryExpression(
                    Operator::Less,
                    Box::new("A".as_var_expr(1, 7)),
                    Box::new(5.as_lit_expr(1, 11)),
                    ExpressionType::Unresolved
                )
                .at_rc(1, 9),
                statements: vec![
                    Statement::Assignment(
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
                TopLevelToken::Statement(Statement::While(ConditionalBlockNode {
                    condition: "A".as_var_expr(2, 15),
                    statements: vec![
                        Statement::Comment(" keep looping".to_string()).at_rc(2, 20),
                        Statement::System.at_rc(3, 13),
                        Statement::Comment(" exit".to_string()).at_rc(3, 20)
                    ]
                }))
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" end of loop".to_string()))
                    .at_rc(4, 20)
            ]
        );
    }

    #[test]
    fn test_wend_without_while() {
        let input = "WEND";
        assert_parser_err!(input, QError::WendWithoutWhile);
    }

    #[test]
    fn test_while_without_wend() {
        let input = r#"
        WHILE X > 0
        PRINT X
        "#;
        assert_parser_err!(input, QError::WhileWithoutWend);
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
            Statement::While(ConditionalBlockNode {
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
            QError::syntax_error("Expected: end-of-statement"),
            3,
            21
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
            Statement::While(ConditionalBlockNode {
                condition: Expression::BinaryExpression(
                    Operator::Less,
                    Box::new("A".as_var_expr(2, 15)),
                    Box::new(5.as_lit_expr(2, 19)),
                    ExpressionType::Unresolved
                )
                .at_rc(2, 17),
                statements: vec![
                    Statement::Assignment(
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
