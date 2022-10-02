use crate::common::*;
use crate::parser::expression;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statement_separator::comments_and_whitespace_p;
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
    seq5(
        select_case_expr_p(),
        comments_and_whitespace_p(),
        case_blocks(),
        case_else().allow_none(),
        keyword_pair(Keyword::End, Keyword::Select).no_incomplete(),
        |expr, inline_comments, case_blocks, else_block, _| {
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
    keyword_pair(Keyword::Select, Keyword::Case).then_demand(
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

fn case_blocks() -> impl Parser<Output = Vec<CaseBlockNode>> + NonOptParser {
    case_block().zero_or_more()
}

fn case_block() -> impl Parser<Output = CaseBlockNode> {
    // CASE
    CaseButNotElse.then_demand(
        OptAndPC::new(whitespace(), continue_after_case())
            .keep_right()
            .or_syntax_error("Expected case expression after CASE"),
    )
}

struct CaseButNotElse;

impl Parser for CaseButNotElse {
    type Output = ();
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        match tokenizer.read()? {
            Some(case_token) if Keyword::Case == case_token => match tokenizer.read()? {
                Some(space_token) if space_token.kind == TokenType::Whitespace as i32 => {
                    match tokenizer.read()? {
                        Some(else_token) if Keyword::Else == else_token => {
                            tokenizer.unread(else_token);
                            tokenizer.unread(space_token);
                            tokenizer.unread(case_token);
                            Err(QError::Incomplete)
                        }
                        Some(other_token) => {
                            tokenizer.unread(other_token);
                            Ok(())
                        }
                        None => Err(QError::syntax_error(
                            "Expected: ELSE or expression after CASE",
                        )),
                    }
                }
                Some(paren_token) if paren_token.kind == TokenType::LParen as i32 => {
                    tokenizer.unread(paren_token);
                    Ok(())
                }
                _ => Err(QError::syntax_error(
                    "Expected: whitespace or parenthesis after CASE",
                )),
            },
            Some(token) => {
                tokenizer.unread(token);
                Err(QError::Incomplete)
            }
            None => Err(QError::Incomplete),
        }
    }
}

fn continue_after_case() -> impl Parser<Output = CaseBlockNode> {
    seq2(
        case_expression_list(),
        ZeroOrMoreStatements::new(keyword_choice(&[Keyword::Case, Keyword::End])),
        |expression_list, statements| CaseBlockNode {
            expression_list,
            statements,
        },
    )
}

fn case_expression_list() -> impl Parser<Output = Vec<CaseExpression>> {
    csv(CaseExpressionParser::new())
}

struct CaseExpressionParser;

impl Parser for CaseExpressionParser {
    type Output = CaseExpression;
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        Self::case_is().or(SimpleOrRangeParser::new()).parse(reader)
    }
}

impl CaseExpressionParser {
    fn new() -> Self {
        Self
    }

    fn case_is() -> impl Parser<Output = CaseExpression> {
        seq3(
            keyword(Keyword::Is),
            OptAndPC::new(whitespace(), expression::relational_operator_p())
                .keep_right()
                .or_syntax_error("Expected: Operator after IS"),
            OptAndPC::new(whitespace(), expression::expression_node_p())
                .keep_right()
                .or_syntax_error("Expected: expression after IS operator"),
            |_, Locatable { element, .. }, r| CaseExpression::Is(element, r),
        )
    }
}

struct SimpleOrRangeParser;

impl Parser for SimpleOrRangeParser {
    type Output = CaseExpression;
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let expr = expression::expression_node_p().parse(reader)?;
        let parenthesis = expr.as_ref().is_parenthesis();
        let to_keyword = whitespace_boundary(parenthesis)
            .and(keyword(Keyword::To))
            .parse_opt(reader)?;
        match to_keyword {
            Some(_) => {
                let second_expr = expression::guarded_expression_node_p()
                    .or_syntax_error("Expected: expression after TO")
                    .parse(reader)?;
                Ok(CaseExpression::Range(expr, second_expr))
            }
            None => Ok(CaseExpression::Simple(expr)),
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
        .then_demand(ZeroOrMoreStatements::new(keyword(Keyword::End)))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_parser_err;
    use crate::common::*;
    use crate::parser::types::*;

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

    // TODO move macros up and use them more

    macro_rules! int_lit {
        ($value: literal) => {
            Expression::IntegerLiteral($value)
        };

        ($value: literal at $row: literal:$col: literal) => {
            Locatable::new(int_lit!($value), Location::new($row, $col))
        };
    }

    macro_rules! bin_exp {
        ($left: expr ; plus $right: expr) => {
            Expression::BinaryExpression(
                Operator::Plus,
                Box::new($left),
                Box::new($right),
                ExpressionType::Unresolved,
            )
        };

        ($left: expr ; plus $right: expr ; at $row: literal:$col: literal) => {
            Locatable::new(bin_exp!($left ; plus $right), Location::new($row, $col))
        };
    }

    macro_rules! paren_exp {
        ($child: expr ; at $row: literal:$col: literal) => {
            Locatable::new(
                Expression::Parenthesis(Box::new($child)),
                Location::new($row, $col),
            )
        };
    }

    #[test]
    fn test_parenthesis() {
        let input = "
        SELECT CASE(5+2)
        CASE(6+5)
            PRINT 11
        CASE(2)TO(5)
            PRINT 2
        END SELECT
        ";
        let result = parse(input).demand_single_statement();
        assert_eq!(
            result,
            Statement::SelectCase(SelectCaseNode {
                expr: paren_exp!( bin_exp!( int_lit!(5 at 2:21) ; plus int_lit!(2 at 2:23) ; at 2:22 ) ; at 2:20 ),
                inline_comments: vec![],
                case_blocks: vec![
                    CaseBlockNode {
                        expression_list: vec![CaseExpression::Simple(
                            paren_exp!( bin_exp!( int_lit!(6 at 3:14) ; plus int_lit!(5 at 3:16) ; at 3:15 ) ; at 3:13 )
                        )],
                        statements: vec![Statement::Print(PrintNode {
                            file_number: None,
                            lpt1: false,
                            format_string: None,
                            args: vec![PrintArg::Expression(11.as_lit_expr(4, 19))]
                        })
                        .at_rc(4, 13)]
                    },
                    CaseBlockNode {
                        expression_list: vec![CaseExpression::Range(
                            Expression::Parenthesis(Box::new(2.as_lit_expr(5, 14))).at_rc(5, 13),
                            Expression::Parenthesis(Box::new(5.as_lit_expr(5, 19))).at_rc(5, 18)
                        )],
                        statements: vec![Statement::Print(PrintNode {
                            file_number: None,
                            lpt1: false,
                            format_string: None,
                            args: vec![PrintArg::Expression(2.as_lit_expr(6, 19))]
                        })
                        .at_rc(6, 13)]
                    },
                ],
                else_block: None
            })
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
