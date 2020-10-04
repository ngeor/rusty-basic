use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statements::*;
use crate::parser::types::*;
use std::io::BufRead;

pub fn while_wend<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        seq3(
            parse_while_expression(),
            statements(
                keyword(Keyword::Wend),
                QError::syntax_error_fn("Expected: end-of-statement"),
            ),
            demand(keyword(Keyword::Wend), || QError::WhileWithoutWend),
        ),
        |(l, r, _)| {
            Statement::While(ConditionalBlockNode {
                condition: l,
                statements: r,
            })
        },
    )
}

fn parse_while_expression<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    drop_left(seq2(
        keyword(Keyword::While),
        expression::demand_guarded_expression_node(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::{
        BareName, ConditionalBlockNode, Expression, Operator, Statement, TopLevelToken,
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
                    Box::new(5.as_lit_expr(2, 19))
                )
                .at_rc(2, 17),
                statements: vec![Statement::SubCall(BareName::from("SYSTEM"), vec![]).at_rc(3, 13)]
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
                    Box::new(5.as_lit_expr(1, 11))
                )
                .at_rc(1, 9),
                statements: vec![
                    Statement::Assignment(
                        "A".into(),
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("A".as_var_expr(1, 18)),
                            Box::new(1.as_lit_expr(1, 22))
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
                        Statement::SubCall(BareName::from("SYSTEM"), vec![]).at_rc(3, 13),
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
        assert_eq!(parse_err(input), QError::WendWithoutWhile);
    }

    #[test]
    fn test_while_without_wend() {
        let input = r#"
        WHILE X > 0
        PRINT X
        "#;
        assert_eq!(parse_err(input), QError::WhileWithoutWend);
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
                        Box::new(0.as_lit_expr(2, 19))
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
        assert_eq!(
            parse_err_node(input),
            QErrorNode::Pos(
                QError::syntax_error("Expected: end-of-statement"),
                Location::new(3, 21)
            )
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
                    Box::new(5.as_lit_expr(2, 19))
                )
                .at_rc(2, 17),
                statements: vec![
                    Statement::Assignment(
                        "A".into(),
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("A".as_var_expr(3, 17)),
                            Box::new(1.as_lit_expr(3, 21))
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
