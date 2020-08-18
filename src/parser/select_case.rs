use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::comment;

use crate::parser::expression;
use crate::parser::statements::parse_statements;
use crate::parser::types::*;
use std::io::BufRead;

pub fn take_if_select_case<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    Box::new(|lexer| try_read(lexer).transpose())
}

#[deprecated]
pub fn try_read<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, QErrorNode> {
    if !lexer.peek_ref_ng().is_keyword(Keyword::Select) {
        return Ok(None);
    }
    let pos = lexer.read()?.pos();
    // initial state: we just read the "SELECT" keyword
    read_whitespace(lexer, "Expected space after SELECT")?;
    read_keyword(lexer, Keyword::Case)?;
    read_whitespace(lexer, "Expected space after CASE")?;
    let expr: ExpressionNode = read(
        lexer,
        expression::try_read,
        "Expected expression after SELECT CASE",
    )?;
    let mut inline_comments: Vec<Locatable<String>> = vec![];
    let inline_statements = parse_inline_comments(lexer)?;
    for inline_statement in inline_statements {
        match inline_statement.as_ref() {
            Statement::Comment(c) => {
                inline_comments.push(c.clone().at(inline_statement.pos()));
            }
            _ => panic!("should have been a comment"),
        }
    }

    // TODO support multiple expressions e.g. CASE 1,2,IS<=3
    let mut case_blocks: Vec<CaseBlockNode> = vec![];
    loop {
        match try_read_case(lexer)? {
            Some(c) => case_blocks.push(c),
            None => break,
        }
    }
    let else_block = try_read_case_else(lexer)?;
    read_keyword(lexer, Keyword::End)?;
    read_whitespace(lexer, "Expected space after END")?;
    read_keyword(lexer, Keyword::Select)?;
    Ok(Some(
        Statement::SelectCase(SelectCaseNode {
            inline_comments,
            expr,
            case_blocks,
            else_block,
        })
        .at(pos),
    ))
}

/// This is a trimmed-down version of parse_statements, to parse any comments
/// between SELECT CASE X ... until the first CASE expression
fn parse_inline_comments<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<StatementNodes, QErrorNode> {
    let mut statements: StatementNodes = vec![];

    loop {
        let p = lexer.peek_ref_ng();
        if p.is_keyword(Keyword::Case) || p.is_keyword(Keyword::End) {
            return Ok(statements);
        } else if p.is_whitespace() || p.is_eol() {
            lexer.read_ng()?;
        } else if p.is_symbol('\'') {
            // read comment, regardless of whether we've seen the separator or not
            let s = read(lexer, comment::try_read, "Expected comment")?;
            statements.push(s);
        // Comments do not need an inline separator but they require a EOL/EOF post-separator
        } else {
            return Err(QError::SyntaxError("Expected CASE".to_string())).with_err_at(lexer.pos());
        }
    }
}

fn peek_case_else<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<bool, QErrorNode> {
    let mut found_case_else = false;
    lexer.begin_transaction();
    if lexer.peek_ref_ng().is_keyword(Keyword::Case) {
        lexer.read_ng()?;
        if lexer.peek_ref_ng().is_whitespace() {
            lexer.read_ng()?;
            found_case_else = lexer.peek_ref_ng().is_keyword(Keyword::Else);
        } else {
            // CASE should always be followed by a space so it's okay to throw an error here
            return Err(QError::SyntaxError("Expected space after CASE".to_string()))
                .with_err_at(lexer.pos());
        }
    }
    lexer.rollback_transaction();
    Ok(found_case_else)
}

fn try_read_case<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<CaseBlockNode>, QErrorNode> {
    if !lexer.peek_ref_ng().is_keyword(Keyword::Case) {
        return Ok(None);
    }
    if peek_case_else(lexer)? {
        return Ok(None);
    }
    lexer.read_ng()?; // CASE
    lexer.read_ng()?; // whitespace
    if lexer.peek_ref_ng().is_keyword(Keyword::Is) {
        read_case_is(lexer) // IS
    } else {
        read_case_expr(lexer)
    }
}

fn read_case_is<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<CaseBlockNode>, QErrorNode> {
    lexer.read_ng()?; // IS
    skip_whitespace(lexer)?;
    let op = read_relational_operator(lexer)?;
    skip_whitespace(lexer)?;
    let expr = read(lexer, expression::try_read, "Expected expression after IS")?;
    let statements = parse_statements(
        lexer,
        |x| x.is_keyword(Keyword::Case) || x.is_keyword(Keyword::End),
        "Unterminated CASE IS",
    )?;
    Ok(Some(CaseBlockNode {
        expr: CaseExpression::Is(op, expr),
        statements,
    }))
}

fn read_relational_operator<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Operand, QErrorNode> {
    let next = lexer.read()?;
    if next.as_ref().is_symbol('=') {
        Ok(Operand::Equal)
    } else if next.as_ref().is_symbol('>') {
        if lexer.peek_ref_ng().is_symbol('=') {
            lexer.read_ng()?;
            Ok(Operand::GreaterOrEqual)
        } else {
            Ok(Operand::Greater)
        }
    } else if next.as_ref().is_symbol('<') {
        if lexer.peek_ref_ng().is_symbol('=') {
            lexer.read_ng()?;
            Ok(Operand::LessOrEqual)
        } else if lexer.peek_ref_ng().is_symbol('>') {
            lexer.read_ng()?;
            Ok(Operand::NotEqual)
        } else {
            Ok(Operand::Less)
        }
    } else {
        Err(QError::SyntaxError(
            "Expected relational operator".to_string(),
        ))
        .with_err_at(&next)
    }
}

fn read_case_expr<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<CaseBlockNode>, QErrorNode> {
    let first_expr = read(
        lexer,
        expression::try_read,
        "Expected expression after CASE",
    )?;
    let mut second_expr: Option<ExpressionNode> = None;
    lexer.begin_transaction();
    skip_whitespace(lexer)?;
    if lexer.read()?.as_ref().is_keyword(Keyword::To) {
        lexer.commit_transaction();
        skip_whitespace(lexer)?;
        second_expr =
            read(lexer, expression::try_read, "Expected expression after TO").map(|x| Some(x))?;
    } else {
        lexer.rollback_transaction();
    }
    let statements = parse_statements(
        lexer,
        |x| x.is_keyword(Keyword::Case) || x.is_keyword(Keyword::End),
        "Unterminated CASE",
    )?;
    let case_expr = match second_expr {
        Some(s) => CaseExpression::Range(first_expr, s),
        None => CaseExpression::Simple(first_expr),
    };
    Ok(Some(CaseBlockNode {
        expr: case_expr,
        statements,
    }))
}

fn try_read_case_else<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNodes>, QErrorNode> {
    if !peek_case_else(lexer)? {
        return Ok(None);
    }
    lexer.read_ng()?; // CASE
    lexer.read_ng()?; // whitespace
    lexer.read_ng()?; // ELSE
    let statements = parse_statements(
        lexer,
        |x| x.is_keyword(Keyword::End),
        "Unterminated CASE ELSE",
    )?;
    Ok(Some(statements))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::*;

    #[test]
    fn test_inline_comment() {
        let input = r#"
        SELECT CASE X ' testing for x
        CASE 1        ' is it one?
        PRINT "One"   ' print it
        CASE ELSE     ' something else?
        PRINT "Nope"  ' print nope
        END SELECT    ' end of select
        "#;
        let result = parse(input);
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::SelectCase(SelectCaseNode {
                    expr: "X".as_var_expr(2, 21),
                    inline_comments: vec![" testing for x".to_string().at_rc(2, 23)],
                    case_blocks: vec![CaseBlockNode {
                        expr: CaseExpression::Simple(1.as_lit_expr(3, 14)),
                        statements: vec![
                            Statement::Comment(" is it one?".to_string()).at_rc(3, 23),
                            Statement::SubCall("PRINT".into(), vec!["One".as_lit_expr(4, 15)])
                                .at_rc(4, 9),
                            Statement::Comment(" print it".to_string()).at_rc(4, 23),
                        ]
                    }],
                    else_block: Some(vec![
                        Statement::Comment(" something else?".to_string()).at_rc(5, 23),
                        Statement::SubCall("PRINT".into(), vec!["Nope".as_lit_expr(6, 15)])
                            .at_rc(6, 9),
                        Statement::Comment(" print nope".to_string()).at_rc(6, 23),
                    ]),
                }))
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" end of select".to_string()))
                    .at_rc(7, 23)
            ]
        );
    }

    #[test]
    fn test_no_case() {
        let input = r#"
        SELECT CASE X
        END SELECT
        "#;
        let result = parse(input);
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::SelectCase(SelectCaseNode {
                    expr: "X".as_var_expr(2, 21),
                    inline_comments: vec![],
                    case_blocks: vec![],
                    else_block: None
                }))
                .at_rc(2, 9)
            ]
        );
    }

    #[test]
    fn test_inline_comment_next_line() {
        let input = r#"
        SELECT CASE X ' testing for x
        ' first case
        CASE 1        ' is it one?
        PRINT "One"   ' print it
        END SELECT
        "#;
        let result = parse(input);
        assert_eq!(
            result,
            vec![
                TopLevelToken::Statement(Statement::SelectCase(SelectCaseNode {
                    expr: "X".as_var_expr(2, 21),
                    inline_comments: vec![
                        " testing for x".to_string().at_rc(2, 23),
                        " first case".to_string().at_rc(3, 9)
                    ],
                    case_blocks: vec![CaseBlockNode {
                        expr: CaseExpression::Simple(1.as_lit_expr(4, 14)),
                        statements: vec![
                            Statement::Comment(" is it one?".to_string()).at_rc(4, 23),
                            Statement::SubCall("PRINT".into(), vec!["One".as_lit_expr(5, 15)])
                                .at_rc(5, 9),
                            Statement::Comment(" print it".to_string()).at_rc(5, 23),
                        ]
                    }],
                    else_block: None
                }))
                .at_rc(2, 9)
            ]
        );
    }
}
