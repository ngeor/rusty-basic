use rusty_pc::*;

use crate::input::RcStringView;
use crate::specific::core::expression::ws_expr_pos_p;
use crate::specific::core::statement_separator::comments_and_whitespace_p;
use crate::specific::core::statements::ZeroOrMoreStatements;
use crate::specific::pc_specific::*;
use crate::specific::*;
use crate::ParseError;

// SELECT CASE expr ' comment
// CASE 1
// CASE IS >= 2
// CASE 5 TO 7
// CASE ELSE
// END SELECT

// CASE <ws+> ELSE
// CASE <expr> TO <expr>
// CASE <ws+> IS <Operator> <expr>
// CASE <expr>

pub fn select_case_p() -> impl Parser<RcStringView, Output = Statement, Error = ParseError> {
    seq4(
        select_case_expr_p(),
        comments_and_whitespace_p(),
        case_blocks(),
        keyword_pair(Keyword::End, Keyword::Select),
        |expr, inline_comments, all_case_blocks: Vec<CaseBlock>, _| {
            // TODO 1 do not clone 2 fail if multiple ELSE blocks 3 fail if ELSE block is not the last one
            // TODO revisit this
            let case_blocks = all_case_blocks
                .clone()
                .into_iter()
                .filter(|x| x.has_conditions())
                .collect();
            let else_block = all_case_blocks
                .into_iter()
                .find(|x| !x.has_conditions())
                .map(|x| {
                    let (_, right) = x.into();
                    right
                });
            Statement::SelectCase(SelectCase {
                expr,
                case_blocks,
                else_block,
                inline_comments,
            })
        },
    )
}

/// Parses the `SELECT CASE expression` part
fn select_case_expr_p() -> impl Parser<RcStringView, Output = ExpressionPos, Error = ParseError> {
    keyword_pair(Keyword::Select, Keyword::Case)
        .and_keep_right(ws_expr_pos_p().or_syntax_error("Expected: expression after CASE"))
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

fn case_blocks() -> impl Parser<RcStringView, Output = Vec<CaseBlock>, Error = ParseError> {
    case_block().zero_or_more()
}

fn case_block() -> impl Parser<RcStringView, Output = CaseBlock, Error = ParseError> {
    // CASE
    // TODO is this syntax_error message even possible to happen?
    keyword(Keyword::Case).and_keep_right(
        continue_after_case().or_syntax_error("Expected: 'case expression' or ELSE after CASE"),
    )
}

fn continue_after_case() -> impl Parser<RcStringView, Output = CaseBlock, Error = ParseError> {
    opt_and_keep_right(
        whitespace(),
        seq2(
            OrParser::new(vec![
                Box::new(keyword(Keyword::Else).map(|_| vec![])),
                Box::new(case_expression_list()),
            ]),
            ZeroOrMoreStatements::new_multi(vec![Keyword::Case, Keyword::End]),
            CaseBlock::new,
        ),
    )
}

fn case_expression_list(
) -> impl Parser<RcStringView, Output = Vec<CaseExpression>, Error = ParseError> {
    csv(case_expression_parser::parser())
}

mod case_expression_parser {
    use rusty_common::Positioned;
    use rusty_pc::*;

    use crate::input::RcStringView;
    use crate::specific::core::expression::expression_pos_p;
    use crate::specific::core::opt_second_expression::opt_second_expression_after_keyword;
    use crate::specific::pc_specific::*;
    use crate::specific::{CaseExpression, Keyword, Operator};
    use crate::{ExpressionTrait, ParseError};

    pub fn parser() -> impl Parser<RcStringView, Output = CaseExpression, Error = ParseError> {
        case_is().or(simple_or_range())
    }

    fn case_is() -> impl Parser<RcStringView, Output = CaseExpression, Error = ParseError> {
        seq3(
            keyword(Keyword::Is),
            opt_and_keep_right(whitespace(), relational_operator_p())
                .or_syntax_error("Expected: Operator after IS"),
            opt_and_keep_right(whitespace(), expression_pos_p())
                .or_syntax_error("Expected: expression after IS operator"),
            |_, Positioned { element, .. }, r| CaseExpression::Is(element, r),
        )
    }

    fn relational_operator_p(
    ) -> impl Parser<RcStringView, Output = Positioned<Operator>, Error = ParseError> {
        any_token()
            .filter_map(|token| match TokenType::from_token(token) {
                TokenType::LessEquals => Some(Operator::LessOrEqual),
                TokenType::GreaterEquals => Some(Operator::GreaterOrEqual),
                TokenType::NotEquals => Some(Operator::NotEqual),
                TokenType::Less => Some(Operator::Less),
                TokenType::Greater => Some(Operator::Greater),
                TokenType::Equals => Some(Operator::Equal),
                _ => None,
            })
            .with_pos()
    }

    fn simple_or_range() -> impl Parser<RcStringView, Output = CaseExpression, Error = ParseError> {
        opt_second_expression_after_keyword(
            expression_pos_p(),
            Keyword::To,
            ExpressionTrait::is_parenthesis,
        )
        .map(|(left, opt_right)| match opt_right {
            Some(right) => CaseExpression::Range(left, right),
            _ => CaseExpression::Simple(left),
        })
    }
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::error::ParseError;
    use crate::specific::*;
    use crate::test_utils::*;
    use crate::{assert_parser_err, bin_exp, int_lit, paren_exp, *};
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
                GlobalStatement::Statement(Statement::SelectCase(SelectCase {
                    expr: "X".as_var_expr(2, 21),
                    inline_comments: vec![" testing for x".to_string().at_rc(2, 23)],
                    case_blocks: vec![CaseBlock::new(
                        vec![CaseExpression::Simple(1.as_lit_expr(3, 14))],
                        vec![
                            Statement::Comment(" is it one?".to_string()).at_rc(3, 23),
                            Statement::sub_call("Flint".into(), vec!["One".as_lit_expr(4, 15)])
                                .at_rc(4, 9),
                            Statement::Comment(" print it".to_string()).at_rc(4, 23),
                        ]
                    )],
                    else_block: Some(vec![
                        Statement::Comment(" something else?".to_string()).at_rc(5, 23),
                        Statement::sub_call("Flint".into(), vec!["Nope".as_lit_expr(6, 15)])
                            .at_rc(6, 9),
                        Statement::Comment(" print nope".to_string()).at_rc(6, 23),
                    ]),
                }))
                .at_rc(2, 9),
                GlobalStatement::Statement(Statement::Comment(" end of select".to_string()))
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
                GlobalStatement::Statement(Statement::SelectCase(SelectCase {
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
                GlobalStatement::Statement(Statement::SelectCase(SelectCase {
                    expr: "X".as_var_expr(2, 21),
                    inline_comments: vec![
                        " testing for x".to_string().at_rc(2, 23),
                        " first case".to_string().at_rc(3, 9)
                    ],
                    case_blocks: vec![CaseBlock::new(
                        vec![CaseExpression::Simple(1.as_lit_expr(4, 14))],
                        vec![
                            Statement::Comment(" is it one?".to_string()).at_rc(4, 23),
                            Statement::sub_call("Flint".into(), vec!["One".as_lit_expr(5, 15)])
                                .at_rc(5, 9),
                            Statement::Comment(" print it".to_string()).at_rc(5, 23),
                        ]
                    )],
                    else_block: None
                }))
                .at_rc(2, 9)
            ]
        );
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
            Statement::SelectCase(SelectCase {
                expr: paren_exp!( bin_exp!( int_lit!(5 at 2:21) ; plus int_lit!(2 at 2:23) ; at 2:22 ) ; at 2:20 ),
                inline_comments: vec![],
                case_blocks: vec![
                    CaseBlock::new(
                        vec![CaseExpression::Simple(
                            paren_exp!( bin_exp!( int_lit!(6 at 3:14) ; plus int_lit!(5 at 3:16) ; at 3:15 ) ; at 3:13 )
                        )],
                        vec![Statement::Print(Print {
                            file_number: None,
                            lpt1: false,
                            format_string: None,
                            args: vec![PrintArg::Expression(11.as_lit_expr(4, 19))]
                        })
                        .at_rc(4, 13)]
                    ),
                    CaseBlock::new(
                        vec![CaseExpression::Range(
                            Expression::Parenthesis(Box::new(2.as_lit_expr(5, 14))).at_rc(5, 13),
                            Expression::Parenthesis(Box::new(5.as_lit_expr(5, 19))).at_rc(5, 18)
                        )],
                        vec![Statement::Print(Print {
                            file_number: None,
                            lpt1: false,
                            format_string: None,
                            args: vec![PrintArg::Expression(2.as_lit_expr(6, 19))]
                        })
                        .at_rc(6, 13)]
                    ),
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
        assert_parser_err!(input, ParseError::syntax_error("Expected: CASE"), 2, 16);
    }

    #[test]
    fn test_no_space_after_case() {
        let input = "
        SELECT CASE X
        CASE1
        END SELECT";
        assert_parser_err!(input, ParseError::syntax_error("Expected: END"), 3, 9);
    }

    #[test]
    fn test_no_space_unfinished_to() {
        let input = "
        SELECT CASE X
        CASE 1 TO
        END SELECT";
        assert_parser_err!(
            input,
            ParseError::syntax_error("Expected: expression after TO"),
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
            ParseError::syntax_error("Expected: end-of-statement"),
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
            ParseError::syntax_error("Expected: end-of-statement"),
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
            ParseError::syntax_error("Expected: end-of-statement"),
            3,
            15
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
            ParseError::syntax_error("Expected: end-of-statement"),
            3,
            15
        );
    }
}
