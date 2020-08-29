use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::pc::common::*;
use crate::parser::statements::*;
use crate::parser::types::*;
use std::io::BufRead;

pub fn while_wend<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QError>)> {
    map(
        seq3(
            parse_while_expression(),
            demand(
                statements(try_read_keyword(Keyword::Wend)),
                QError::syntax_error_fn("Expected WHILE statements"),
            ),
            demand(try_read_keyword(Keyword::Wend), || QError::WhileWithoutWend),
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
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
    drop_left(crate::parser::pc::ws::seq2(
        try_read_keyword(Keyword::While),
        demand(
            expression::expression_node(),
            QError::syntax_error_fn("Expected WHILE condition"),
        ),
        // TODO is WHILE(X>1) acceptable in QB?
        QError::syntax_error_fn("Expected whitespace after WHILE"),
    ))
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::{
        BareName, ConditionalBlockNode, Expression, Operand, Statement, TopLevelToken,
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
                    Operand::Less,
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
        let program = parse("WHILE A < 5: A = A + 1: PRINT A: WEND").demand_single_statement();
        assert_eq!(
            program,
            Statement::While(ConditionalBlockNode {
                condition: Expression::BinaryExpression(
                    Operand::Less,
                    Box::new("A".as_var_expr(1, 7)),
                    Box::new(5.as_lit_expr(1, 11))
                )
                .at_rc(1, 9),
                statements: vec![
                    Statement::Assignment(
                        "A".into(),
                        Expression::BinaryExpression(
                            Operand::Plus,
                            Box::new("A".as_var_expr(1, 18)),
                            Box::new(1.as_lit_expr(1, 22))
                        )
                        .at_rc(1, 20)
                    )
                    .at_rc(1, 14),
                    Statement::SubCall(BareName::from("PRINT"), vec!["A".as_var_expr(1, 31)])
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
}
