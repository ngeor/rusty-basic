use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::comment;
use crate::parser::error::*;
use crate::parser::expression;
use crate::parser::statements::parse_statements;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    if !lexer.peek()?.is_keyword(Keyword::Select) {
        return Ok(None);
    }
    let pos = lexer.read()?.location();
    // initial state: we just read the "SELECT" keyword
    read_demand_whitespace(lexer, "Expected space after SELECT")?;
    read_demand_keyword(lexer, Keyword::Case)?;
    read_demand_whitespace(lexer, "Expected space after CASE")?;
    let expr: ExpressionNode = demand(
        lexer,
        expression::try_read,
        "Expected expression after SELECT CASE",
    )?;
    let mut inline_comments: Vec<Locatable<String>> = vec![];
    let inline_statements = parse_inline_comments(lexer)?;
    for inline_statement in inline_statements {
        match inline_statement.as_ref() {
            Statement::Comment(c) => {
                inline_comments.push(c.clone().at(inline_statement.location()));
            }
            _ => panic!("should have been a comment"),
        }
    }

    // TODO what if there is a comment on the next line between SELECT CASE and the first CASE
    // TODO what if there is no CASE inside SELECT
    // TODO support multiple expressions e.g. CASE 1,2,IS<=3
    let mut case_blocks: Vec<CaseBlockNode> = vec![];
    loop {
        match try_read_case(lexer)? {
            Some(c) => case_blocks.push(c),
            None => break,
        }
    }
    let else_block = try_read_case_else(lexer)?;
    read_demand_keyword(lexer, Keyword::End)?;
    read_demand_whitespace(lexer, "Expected space after END")?;
    read_demand_keyword(lexer, Keyword::Select)?;
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
fn parse_inline_comments<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<StatementNodes, ParserError> {
    let mut statements: StatementNodes = vec![];

    loop {
        let p = lexer.peek()?;
        if p.is_keyword(Keyword::Case) {
            return Ok(statements);
        } else if p.is_eof() {
            return unexpected("Expected CASE", p);
        } else if p.is_whitespace() || p.is_eol() {
            lexer.read()?;
        } else if p.is_symbol('\'') {
            // read comment, regardless of whether we've seen the separator or not
            // TODO add unit test where comment reads EOF
            let s = demand(lexer, comment::try_read, "Expected comment")?;
            statements.push(s);
        // Comments do not need an inline separator but they require a EOL/EOF post-separator
        } else {
            return Err(ParserError::Unterminated(p));
        }
    }
}

fn peek_case_else<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<bool, ParserError> {
    let mut found_case_else = false;
    lexer.begin_transaction();
    if lexer.peek()?.is_keyword(Keyword::Case) {
        lexer.read()?;
        if lexer.peek()?.is_whitespace() {
            lexer.read()?;
            found_case_else = lexer.peek()?.is_keyword(Keyword::Else);
        } else {
            // CASE should always be followed by a space so it's okay to throw an error here
            return Err(ParserError::SyntaxError(
                "Expected space after CASE".to_string(),
                lexer.peek()?.location(),
            ));
        }
    }
    lexer.rollback_transaction()?;
    Ok(found_case_else)
}

fn try_read_case<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<CaseBlockNode>, ParserError> {
    if !lexer.peek()?.is_keyword(Keyword::Case) {
        return Ok(None);
    }
    if peek_case_else(lexer)? {
        return Ok(None);
    }
    lexer.read()?; // CASE
    lexer.read()?; // whitespace
    if lexer.peek()?.is_keyword(Keyword::Is) {
        read_case_is(lexer) // IS
    } else {
        read_case_expr(lexer)
    }
}

fn read_case_is<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<CaseBlockNode>, ParserError> {
    lexer.read()?; // IS
    skip_whitespace(lexer)?;
    let op = read_relational_operator(lexer)?;
    skip_whitespace(lexer)?;
    let expr = demand(lexer, expression::try_read, "Expected expression after IS")?;
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

fn read_relational_operator<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Operand, ParserError> {
    let next = lexer.read()?;
    if next.is_symbol('=') {
        Ok(Operand::Equal)
    } else if next.is_symbol('>') {
        if lexer.peek()?.is_symbol('=') {
            lexer.read()?;
            Ok(Operand::GreaterOrEqual)
        } else {
            Ok(Operand::Greater)
        }
    } else if next.is_symbol('<') {
        if lexer.peek()?.is_symbol('=') {
            lexer.read()?;
            Ok(Operand::LessOrEqual)
        } else if lexer.peek()?.is_symbol('>') {
            lexer.read()?;
            Ok(Operand::NotEqual)
        } else {
            Ok(Operand::Less)
        }
    } else {
        Err(ParserError::SyntaxError(
            "Expected relational operator".to_string(),
            next.location(),
        ))
    }
}

fn read_case_expr<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<CaseBlockNode>, ParserError> {
    let first_expr = demand(
        lexer,
        expression::try_read,
        "Expected expression after CASE",
    )?;
    let mut second_expr: Option<ExpressionNode> = None;
    lexer.begin_transaction();
    skip_whitespace(lexer)?;
    if lexer.read()?.is_keyword(Keyword::To) {
        lexer.commit_transaction()?;
        skip_whitespace(lexer)?;
        second_expr =
            demand(lexer, expression::try_read, "Expected expression after TO").map(|x| Some(x))?;
    } else {
        lexer.rollback_transaction()?;
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

fn try_read_case_else<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNodes>, ParserError> {
    if !peek_case_else(lexer)? {
        return Ok(None);
    }
    lexer.read()?; // CASE
    lexer.read()?; // whitespace
    lexer.read()?; // ELSE
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
}
