use crate::common::*;
use crate::parser::base::and_pc::AndDemandTrait;
use crate::parser::base::parsers::{AndOptTrait, KeepRightTrait, ManyTrait, OrTrait, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::specific::{
    demand_keyword_pair_p, keyword_p, keyword_pair_p, whitespace, LeadingWhitespace,
    OrSyntaxErrorTrait,
};
use crate::parser::statements;
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
        .and_opt(comment::comments_and_whitespace_p())
        .and_opt(case_blocks())
        .and_opt(case_else())
        .and_demand(demand_keyword_pair_p(Keyword::End, Keyword::Select))
        .map(
            |((((expr, opt_inline_comments), opt_blocks), else_block), _)| {
                Statement::SelectCase(SelectCaseNode {
                    expr,
                    case_blocks: opt_blocks.unwrap_or_default(),
                    else_block,
                    inline_comments: opt_inline_comments.unwrap_or_default(),
                })
            },
        )
}

/// Parses the `SELECT CASE expression` part
fn select_case_expr_p() -> impl Parser<Output = ExpressionNode> {
    keyword_pair_p(Keyword::Select, Keyword::Case)
        .and_demand(
            expression::guarded_expression_node_p()
                .or_syntax_error("Expected: expression after CASE"),
        )
        .keep_right()
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

fn case_blocks() -> impl Parser<Output = Vec<CaseBlockNode>> {
    CaseBlockParser::new().one_or_more()
}

struct CaseBlockParser;

impl Parser for CaseBlockParser {
    type Output = CaseBlockNode;

    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        // CASE
        let (reader, result) = keyword_p(Keyword::Case)
            .and_opt(whitespace())
            .map(|((_, l), r)| {
                let mut temp: String = String::new();
                temp.push_str(l.as_str());
                temp.push_str(r.unwrap_or_default().as_str());
                temp
            })
            .parse(reader)?;
        if result.is_none() {
            return Ok((reader, None));
        }
        let case_ws_str: String = result.unwrap_or_default();
        let (reader, result) = Self::continue_after_case().parse(reader)?;
        if result.is_some() {
            Ok((reader, result))
        } else {
            Ok((reader.undo(case_ws_str), result))
        }
    }
}

impl CaseBlockParser {
    fn new() -> Self {
        Self
    }

    fn continue_after_case() -> impl Parser<Output = CaseBlockNode> {
        case_expression_list()
            .and_demand(statements::zero_or_more_statements_p(
                keyword_p(Keyword::Case).or(keyword_p(Keyword::End)),
            ))
            .map(|(expression_list, statements)| CaseBlockNode {
                expression_list,
                statements,
            })
    }
}

fn case_expression_list() -> impl Parser<Output = Vec<CaseExpression>> {
    CaseExpressionParser::new().csv()
}

struct CaseExpressionParser;

impl Parser for CaseExpressionParser {
    type Output = CaseExpression;

    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let (reader, result) = keyword_p(Keyword::Else).peek().parse(reader)?;
        if result.is_some() {
            return Ok(None);
        }

        Self::case_is()
            .or(SimpleOrRangeParser::new())
            .or_syntax_error("Expected: IS or expression")
            .parse(reader)
    }
}

impl CaseExpressionParser {
    fn new() -> Self {
        Self
    }

    fn case_is() -> impl Parser<Output = CaseExpression> {
        keyword_p(Keyword::Is)
            .and_opt(whitespace())
            .and_demand(
                expression::relational_operator_p().or_syntax_error("Expected: Operator after IS"),
            )
            .and_opt(whitespace())
            .and_demand(
                expression::expression_node_p()
                    .or_syntax_error("Expected: expression after IS operator"),
            )
            .map(|(((_, op), _), r)| CaseExpression::Is(op.strip_location(), r))
    }
}

struct SimpleOrRangeParser;

impl Parser for SimpleOrRangeParser {
    type Output = CaseExpression;

    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let (reader, expr) = expression::expression_node_p().parse(reader)?;
        match expr {
            Some(expr) => {
                let parenthesis = expr.is_parenthesis();
                let result =
                    LeadingWhitespace::new(keyword_p(Keyword::To), !parenthesis).parse(reader)?;
                match result {
                    Some(_) => {
                        let (reader, second_expr) = expression::guarded_expression_node_p()
                            .or_syntax_error("Expected: expression after TO")
                            .parse(reader)?;
                        Ok((
                            reader,
                            Some(CaseExpression::Range(expr, second_expr.unwrap())),
                        ))
                    }
                    None => Ok((reader, Some(CaseExpression::Simple(expr)))),
                }
            }
            _ => Ok((reader, None)),
        }
    }
}

impl SimpleOrRangeParser {
    fn new() -> Self {
        Self
    }
}

fn case_else() -> impl Parser<Output = StatementNodes> {
    keyword_pair_p(Keyword::Case, Keyword::Else)
        .and_demand(statements::zero_or_more_statements_p(keyword_p(
            Keyword::End,
        )))
        .keep_right()
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
        assert_parser_err!(
            input,
            QError::syntax_error("Expected: CASE after SELECT"),
            2,
            16
        );
    }

    #[test]
    fn test_no_space_after_case() {
        let input = "
        SELECT CASE X
        CASE1
        END SELECT";
        assert_parser_err!(input, QError::syntax_error("Expected: END SELECT"), 3, 9);
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
