use crate::common::*;
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statements::ZeroOrMoreStatements;
use crate::parser::types::*;

// SELECT CASE expr ' comment
// CASE 1
// CASE IS >= 2
// CASE 5 TO 7
// CASE ELSE
// END SELECT

// CASE <ws+> ELSE (priority)
// CASE <expr> TO <expr>
// CASE <ws+> IS <Operator> <expr>
// CASE <expr>

pub fn select_case_p() -> impl Parser<Output = Statement> {
    select_case_expr_p()
        .and_demand(comment::comments_and_whitespace_p())
        .and_demand(case_blocks())
        .and_opt(case_else())
        .and_demand(keyword_pair(Keyword::End, Keyword::Select))
        .map(
            |((((expr, inline_comments), case_blocks), else_block), _)| {
                Statement::SelectCase(SelectCaseNode {
                    expr,
                    case_blocks,
                    else_block,
                    inline_comments,
                })
            },
        )
}

/// Parses the `SELECT CASE expression` part
fn select_case_expr_p() -> impl Parser<Output = ExpressionNode> {
    keyword_pair(Keyword::Select, Keyword::Case).then_use(
        expression::guarded_expression_node_p().or_syntax_error("Expected: expression after CASE"),
    )
}

// SELECT CASE expr
// ' comments and whitespace...
// [CASE case-expression-list
// statements]*
// [CASE ELSE
// statements]?
// END SELECT
//
// case-expression-list := case-expression [, case-expression ]*
// case-expression := is-expression | range-expression | expression
// is-expression := IS rel-op expression
// range-expression := expression TO expression
//
// For case-expression-list, the first element needs to be "guarded" (preceded by whitespace or parenthesis)
// but the remaining elements are already guarded by comma.
//
// For range-expression, no space is needed before TO if the first expression is in parenthesis

fn case_blocks() -> impl NonOptParser<Output = Vec<CaseBlockNode>> {
    case_block().zero_or_more()
}

fn case_block() -> impl Parser<Output = CaseBlockNode> {
    // CASE
    CaseButNotElse.then_use(
        continue_after_case()
            .preceded_by_opt_ws()
            .or_syntax_error("Expected case expression after CASE"),
    )
}

struct CaseButNotElse;

impl HasOutput for CaseButNotElse {
    type Output = ();
}

impl Parser for CaseButNotElse {
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match tokenizer.read()? {
            Some(case_token) if Keyword::Case == case_token => match tokenizer.read()? {
                Some(space_token) if space_token.kind == TokenType::Whitespace as i32 => {
                    match tokenizer.read()? {
                        Some(else_token) if Keyword::Else == else_token => {
                            tokenizer.unread(else_token);
                            tokenizer.unread(space_token);
                            tokenizer.unread(case_token);
                            Ok(None)
                        }
                        Some(other_token) => {
                            tokenizer.unread(other_token);
                            Ok(Some(()))
                        }
                        None => Err(QError::syntax_error(
                            "Expected: ELSE or expression after CASE",
                        )),
                    }
                }
                Some(paren_token) if paren_token.kind == TokenType::LParen as i32 => {
                    tokenizer.unread(paren_token);
                    Ok(Some(()))
                }
                _ => Err(QError::syntax_error(
                    "Expected: whitespace or parenthesis after CASE",
                )),
            },
            Some(token) => {
                tokenizer.unread(token);
                Ok(None)
            }
            None => Ok(None),
        }
    }
}

fn continue_after_case() -> impl Parser<Output = CaseBlockNode> {
    case_expression_list()
        .and_demand(ZeroOrMoreStatements::new(keyword_choice(&[
            Keyword::Case,
            Keyword::End,
        ])))
        .map(|(expression_list, statements)| CaseBlockNode {
            expression_list,
            statements,
        })
}

fn case_expression_list() -> impl Parser<Output = Vec<CaseExpression>> {
    csv_one_or_more(CaseExpressionParser::new())
}

struct CaseExpressionParser;

impl HasOutput for CaseExpressionParser {
    type Output = CaseExpression;
}

impl Parser for CaseExpressionParser {
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        Self::case_is().or(SimpleOrRangeParser::new()).parse(reader)
    }
}

impl CaseExpressionParser {
    fn new() -> Self {
        Self
    }

    fn case_is() -> impl Parser<Output = CaseExpression> {
        keyword(Keyword::Is)
            .and_demand(
                expression::relational_operator_p()
                    .preceded_by_opt_ws()
                    .or_syntax_error("Expected: Operator after IS"),
            )
            .and_demand(
                expression::expression_node_p()
                    .preceded_by_opt_ws()
                    .or_syntax_error("Expected: expression after IS operator"),
            )
            .map(|((_, op), r)| CaseExpression::Is(op.strip_location(), r))
    }
}

struct SimpleOrRangeParser;

impl HasOutput for SimpleOrRangeParser {
    type Output = CaseExpression;
}

impl Parser for SimpleOrRangeParser {
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        match expression::expression_node_p().parse(reader)? {
            Some(expr) => {
                let parenthesis = expr.is_parenthesis();
                let to_keyword = keyword(Keyword::To)
                    .preceded_by_ws(!parenthesis)
                    .parse(reader)?;
                match to_keyword {
                    Some(_) => {
                        let second_expr = expression::guarded_expression_node_p()
                            .or_syntax_error("Expected: expression after TO")
                            .parse_non_opt(reader)?;
                        Ok(Some(CaseExpression::Range(expr, second_expr)))
                    }
                    None => Ok(Some(CaseExpression::Simple(expr))),
                }
            }
            _ => Ok(None),
        }
    }
}

impl SimpleOrRangeParser {
    fn new() -> Self {
        Self
    }
}

fn case_else() -> impl Parser<Output = StatementNodes> {
    keyword_pair(Keyword::Case, Keyword::Else)
        .then_use(ZeroOrMoreStatements::new(keyword(Keyword::End)))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::common::*;
    use crate::parser::types::*;

    use super::super::test_utils::*;

    #[test]
    fn test_select_case_inline_comment() {
        let input = r#"
        SELECT CASE X ' testing for x
        CASE 1        ' is it one?
        Flint "One"   ' print it
        CASE ELSE     ' something else?
        Flint "Nope"  ' print nope
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
                        expression_list: vec![CaseExpression::Simple(1.as_lit_expr(3, 14))],
                        statements: vec![
                            Statement::Comment(" is it one?".to_string()).at_rc(3, 23),
                            Statement::SubCall("Flint".into(), vec!["One".as_lit_expr(4, 15)])
                                .at_rc(4, 9),
                            Statement::Comment(" print it".to_string()).at_rc(4, 23),
                        ]
                    }],
                    else_block: Some(vec![
                        Statement::Comment(" something else?".to_string()).at_rc(5, 23),
                        Statement::SubCall("Flint".into(), vec!["Nope".as_lit_expr(6, 15)])
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
        Flint "One"   ' print it
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
                        expression_list: vec![CaseExpression::Simple(1.as_lit_expr(4, 14))],
                        statements: vec![
                            Statement::Comment(" is it one?".to_string()).at_rc(4, 23),
                            Statement::SubCall("Flint".into(), vec!["One".as_lit_expr(5, 15)])
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

    #[test]
    fn test_no_space_after_select_case() {
        let input = "
        SELECT CASE1
        END SELECT";
        assert_parser_err!(input, QError::syntax_error("Expected: CASE"), 2, 16);
    }

    #[test]
    fn test_no_space_after_case() {
        let input = "
        SELECT CASE X
        CASE1
        END SELECT";
        assert_parser_err!(input, QError::syntax_error("Expected: END"), 3, 9);
    }

    #[test]
    fn test_no_space_unfinished_to() {
        let input = "
        SELECT CASE X
        CASE 1 TO
        END SELECT";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: expression after TO"),
            3,
            18
        );
    }

    #[test]
    fn test_no_space_before_to_unfinished_to() {
        let input = "
        SELECT CASE X
        CASE 1TO
        END SELECT";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: end-of-statement"),
            3,
            15
        );
    }

    #[test]
    fn test_no_space_around_to() {
        let input = "
        SELECT CASE X
        CASE 1TO2
        END SELECT";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: end-of-statement"),
            3,
            15
        );
    }

    #[test]
    fn test_no_space_after_to() {
        let input = "
        SELECT CASE X
        CASE 1 TO2
        END SELECT";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: end-of-statement"),
            3,
            16
        );
    }

    #[test]
    fn test_no_space_before_to() {
        let input = "
        SELECT CASE X
        CASE 1TO 2
        END SELECT";
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: end-of-statement"),
            3,
            15
        );
    }
}
