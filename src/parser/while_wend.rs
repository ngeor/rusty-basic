use super::{ConditionalBlockNode, Statement, StatementNode};
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::expression;
use crate::parser::statements::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, QErrorNode> {
    // TODO lexer consume_if + map
    if !lexer.peek_ref_ng().is_keyword(Keyword::While) {
        return Ok(None);
    }
    let pos = lexer.read()?.pos();
    read_whitespace(lexer, "Expected whitespace after WHILE keyword")?;
    let condition = read(
        lexer,
        expression::try_read,
        "Expected expression after WHILE",
    )?;
    let statements = parse_statements_with_options(
        lexer,
        |x| x.is_keyword(Keyword::Wend),
        ParseStatementsOptions {
            first_statement_separated_by_whitespace: false,
            err: QError::WhileWithoutWend,
        },
    )?;
    lexer.read_ng()?; // read WEND
    Ok(Some(
        Statement::While(ConditionalBlockNode {
            condition,
            statements,
        })
        .at(pos),
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
    fn test_while_wend() {
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
