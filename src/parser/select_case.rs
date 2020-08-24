use crate::char_reader::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

// SELECT CASE expr ' comment
// CASE 1
// CASE IS >= 2
// CASE 5 TO 7
// CASE ELSE
// END SELECT

pub fn select_case<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    map_ng(
        with_keyword_after(
            with_keyword_after(
                if_first_maybe_second(
                    if_first_maybe_second(
                        if_first_maybe_second(
                            with_two_keywords(
                                Keyword::Select,
                                Keyword::Case,
                                expression::expression_node(),
                            ),
                            // parse inline comments after SELECT
                            statements::statements(read_keyword_if(|k| {
                                k == Keyword::Case || k == Keyword::End
                            })),
                        ),
                        case_blocks(),
                    ),
                    case_else(),
                ),
                Keyword::End,
                || QError::SyntaxError("Expected END".to_string()),
            ),
            Keyword::Select,
            || QError::SyntaxError("Expected SELECT".to_string()),
        ),
        |(((expr, inline_statements), opt_blocks), opt_else)| {
            Statement::SelectCase(SelectCaseNode {
                expr,
                case_blocks: opt_blocks.unwrap_or_default(),
                else_block: opt_else,
                inline_comments: inline_statements
                    .unwrap_or_default()
                    .into_iter()
                    .map(|x| match x {
                        Locatable {
                            element: Statement::Comment(text),
                            pos,
                        } => Locatable::new(text, pos),
                        _ => panic!("only comments are allowed - todo improve this"),
                    })
                    .collect(),
            })
        },
    )
}

pub fn case_else<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QErrorNode>)> {
    map_ng(
        with_keyword_before(
            Keyword::Case,
            and_ng(
                try_read_keyword(Keyword::Else),
                statements::statements(try_read_keyword(Keyword::End)),
            ),
        ),
        |(_, r)| r,
    )
}

pub fn case_blocks<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Vec<CaseBlockNode>, QErrorNode>)> {
    take_zero_or_more(case_block(), |_| false)
}

pub fn case_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<CaseBlockNode, QErrorNode>)> {
    map_ng(
        if_first_demand_second(
            case_expr(),
            statements::statements(read_keyword_if(|k| k == Keyword::Case || k == Keyword::End)),
            || QError::SyntaxError("Expected statements after case expression".to_string()),
        ),
        |(expr, statements)| CaseBlockNode { expr, statements },
    )
}

pub fn case_expr<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<CaseExpression, QErrorNode>)> {
    map_ng(
        and_ng(
            try_read_keyword(Keyword::Case),
            and_ng(
                read_any_whitespace(),
                abort_if(
                    try_read_keyword(Keyword::Else),
                    or_ng(case_expr_is(), case_expr_to_or_simple()),
                ),
            ),
        ),
        |(_, (_, r))| r,
    )
}

pub fn case_expr_is<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<CaseExpression, QErrorNode>)> {
    map_ng(
        if_first_demand_second(
            try_read_keyword(Keyword::Is),
            if_first_demand_second(
                read_any_whitespace(),
                if_first_demand_second(
                    expression::operand(false),
                    skipping_whitespace_ng(expression::single_expression_node()),
                    || QError::SyntaxError("Expected expression".to_string()),
                ),
                || QError::SyntaxError("Expected whitespace".to_string()),
            ),
            || QError::SyntaxError("Expected operand".to_string()),
        ),
        |(_, (_, (op, r)))| CaseExpression::Is(op.strip_location(), r),
    )
}

pub fn case_expr_to_or_simple<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<CaseExpression, QErrorNode>)> {
    map_ng(
        if_first_maybe_second(
            expression::expression_node(),
            if_first_demand_second(
                // TODO should be demanding_whitespace_around
                skipping_whitespace_around(try_read_keyword(Keyword::To)),
                expression::expression_node(),
                || QError::SyntaxError("Expected expression".to_string()),
            ),
        ),
        |(l, opt_r)| match opt_r {
            Some((_, r)) => CaseExpression::Range(l, r),
            None => CaseExpression::Simple(l),
        },
    )
}

#[deprecated]
pub fn take_if_select_case<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    Box::new(|lexer| try_read(lexer).transpose())
}

#[deprecated]
pub fn try_read<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, QErrorNode> {
    if !lexer.peek_ref_dp().is_keyword(Keyword::Select) {
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
#[deprecated]
fn parse_inline_comments<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<StatementNodes, QErrorNode> {
    let mut statements: StatementNodes = vec![];

    loop {
        let p = lexer.peek_ref_dp();
        if p.is_keyword(Keyword::Case) || p.is_keyword(Keyword::End) {
            return Ok(statements);
        } else if p.is_whitespace() || p.is_eol() {
            lexer.read_dp()?;
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

#[deprecated]
fn peek_case_else<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<bool, QErrorNode> {
    let mut found_case_else = false;
    lexer.begin_transaction();
    if lexer.peek_ref_dp().is_keyword(Keyword::Case) {
        lexer.read_dp()?;
        if lexer.peek_ref_dp().is_whitespace() {
            lexer.read_dp()?;
            found_case_else = lexer.peek_ref_dp().is_keyword(Keyword::Else);
        } else {
            // CASE should always be followed by a space so it's okay to throw an error here
            return Err(QError::SyntaxError("Expected space after CASE".to_string()))
                .with_err_at(lexer.pos());
        }
    }
    lexer.rollback_transaction();
    Ok(found_case_else)
}

#[deprecated]
fn try_read_case<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<CaseBlockNode>, QErrorNode> {
    if !lexer.peek_ref_dp().is_keyword(Keyword::Case) {
        return Ok(None);
    }
    if peek_case_else(lexer)? {
        return Ok(None);
    }
    lexer.read_dp()?; // CASE
    lexer.read_dp()?; // whitespace
    if lexer.peek_ref_dp().is_keyword(Keyword::Is) {
        read_case_is(lexer) // IS
    } else {
        read_case_expr(lexer)
    }
}

#[deprecated]
fn read_case_is<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<CaseBlockNode>, QErrorNode> {
    lexer.read_dp()?; // IS
    skip_whitespace(lexer)?;
    let op = read_relational_operator(lexer)?;
    skip_whitespace(lexer)?;
    let expr = read(lexer, expression::try_read, "Expected expression after IS")?;
    let statements = statements::parse_statements(
        lexer,
        |x| x.is_keyword(Keyword::Case) || x.is_keyword(Keyword::End),
        "Unterminated CASE IS",
    )?;
    Ok(Some(CaseBlockNode {
        expr: CaseExpression::Is(op, expr),
        statements,
    }))
}

#[deprecated]
fn read_relational_operator<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Operand, QErrorNode> {
    let next = lexer.read()?;
    if next.as_ref().is_symbol('=') {
        Ok(Operand::Equal)
    } else if next.as_ref().is_symbol('>') {
        if lexer.peek_ref_dp().is_symbol('=') {
            lexer.read_dp()?;
            Ok(Operand::GreaterOrEqual)
        } else {
            Ok(Operand::Greater)
        }
    } else if next.as_ref().is_symbol('<') {
        if lexer.peek_ref_dp().is_symbol('=') {
            lexer.read_dp()?;
            Ok(Operand::LessOrEqual)
        } else if lexer.peek_ref_dp().is_symbol('>') {
            lexer.read_dp()?;
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

#[deprecated]
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
    let statements = statements::parse_statements(
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

#[deprecated]
fn try_read_case_else<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNodes>, QErrorNode> {
    if !peek_case_else(lexer)? {
        return Ok(None);
    }
    lexer.read_dp()?; // CASE
    lexer.read_dp()?; // whitespace
    lexer.read_dp()?; // ELSE
    let statements = statements::parse_statements(
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
