use crate::common::*;
use crate::parser::comment;
use crate::parser::expression;
use crate::parser::pc::binary::BinaryParser;
use crate::parser::pc::many::ManyParser;
use crate::parser::pc::text::{whitespace_p, Whitespace};
use crate::parser::pc::unary::UnaryParser;
use crate::parser::pc::unary_fn::UnaryFnParser;
use crate::parser::pc::{static_none_p, Parser, Reader};
use crate::parser::pc_specific::*;
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

pub fn select_case_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    select_case_expr_p()
        .and_opt(comment::comments_and_whitespace_p())
        .and_opt(case_any_many())
        .and_demand(end_select_p().or_syntax_error("Expected: END SELECT"))
        .map(|(((expr, inline_comments), r), _)| {
            let mut case_blocks: Vec<CaseBlockNode> = vec![];
            let mut else_block: Option<StatementNodes> = None;
            for (opt_case_expr, s) in r.unwrap_or_default() {
                match opt_case_expr {
                    ExprOrElse::Expr(case_expr) => {
                        case_blocks.push(CaseBlockNode {
                            expr: case_expr,
                            statements: s,
                        });
                    }
                    ExprOrElse::Else => {
                        if else_block.is_some() {
                            panic!("Multiple case else blocks");
                        }
                        else_block = Some(s);
                    }
                }
            }
            Statement::SelectCase(SelectCaseNode {
                expr,
                case_blocks,
                else_block,
                inline_comments: inline_comments.unwrap_or_default(),
            })
        })
}

fn select_case_expr_p<R>() -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Select)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after SELECT"))
        .and_demand(keyword_p(Keyword::Case).or_syntax_error("Expected: CASE after SELECT"))
        .and_demand(
            expression::guarded_expression_node_p()
                .or_syntax_error("Expected: expression after CASE"),
        )
        .keep_right()
}

enum ExprOrElse {
    Expr(CaseExpression),
    Else,
}

fn case_any_many<R>() -> impl Parser<R, Output = Vec<(ExprOrElse, StatementNodes)>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    case_any().one_or_more_looking_back(|(expr_or_else, _)| match expr_or_else {
        ExprOrElse::Expr(_) => {
            // might have more
            case_any().box_dyn()
        }
        ExprOrElse::Else => {
            // it was the last one
            static_none_p().box_dyn()
        }
    })
}

fn case_any<R>() -> impl Parser<R, Output = (ExprOrElse, StatementNodes)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Case)
        .and_demand(
            case_else()
                .box_dyn()
                .and_demand(statements::zero_or_more_statements_p(keyword_p(
                    Keyword::Else,
                )))
                .or(case_is()
                    .box_dyn()
                    .and_demand(statements::zero_or_more_statements_p(
                        keyword_p(Keyword::Case).or(keyword_p(Keyword::End)),
                    )))
                .or(simple_or_range_p().box_dyn().and_demand(
                    statements::zero_or_more_statements_p(
                        keyword_p(Keyword::Case).or(keyword_p(Keyword::End)),
                    ),
                ))
                .or_syntax_error("Expected: ELSE, IS, expression"),
        )
        .keep_right()
}

fn case_else<R>() -> impl Parser<R, Output = ExprOrElse>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(keyword_p(Keyword::Else))
        .map(|_| ExprOrElse::Else)
}

fn case_is<R>() -> impl Parser<R, Output = ExprOrElse>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    whitespace_p()
        .and(keyword_p(Keyword::Is))
        .and_opt(whitespace_p())
        .and_demand(
            expression::relational_operator_p().or_syntax_error("Expected: Operator after IS"),
        )
        .and_opt(whitespace_p())
        .and_demand(
            expression::expression_node_p()
                .or_syntax_error("Expected: expression after IS operator"),
        )
        .map(|(((_, op), _), r)| ExprOrElse::Expr(CaseExpression::Is(op.strip_location(), r)))
}

fn simple_or_range_p<R>() -> impl Parser<R, Output = ExprOrElse>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    expression::guarded_expression_node_p()
        .and_opt_factory(range_p)
        .map(|(l, opt_r)| match opt_r {
            Some(r) => ExprOrElse::Expr(CaseExpression::Range(l, r)),
            _ => ExprOrElse::Expr(CaseExpression::Simple(l)),
        })
}

fn range_p<R>(first_expr_ref: &ExpressionNode) -> impl Parser<R, Output = ExpressionNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    let parenthesis = first_expr_ref.is_parenthesis();
    Whitespace::new(!parenthesis)
        .and(keyword_p(Keyword::To))
        .and_demand(
            expression::guarded_expression_node_p()
                .or_syntax_error("Expected: expression after TO"),
        )
        .keep_right()
}

fn end_select_p<R>() -> impl Parser<R, Output = ()>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::End)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after END"))
        .and_demand(keyword_p(Keyword::Select).or_syntax_error("Expected: SELECT"))
        .map(|_| ())
}

#[cfg(test)]
mod tests {
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
                        expr: CaseExpression::Simple(1.as_lit_expr(3, 14)),
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
                        expr: CaseExpression::Simple(1.as_lit_expr(4, 14)),
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
        let result = parse_err_node(input);
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: CASE after SELECT"),
                Location::new(2, 16)
            )
        );
    }

    #[test]
    fn test_no_space_after_case() {
        let input = "
        SELECT CASE X
        CASE1
        END SELECT";
        let result = parse_err_node(input);
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: END SELECT"),
                Location::new(3, 9)
            )
        );
    }

    #[test]
    fn test_no_space_unfinished_to() {
        let input = "
        SELECT CASE X
        CASE 1 TO
        END SELECT";
        let result = parse_err_node(input);
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: expression after TO"),
                Location::new(3, 18)
            )
        );
    }

    #[test]
    fn test_no_space_before_to_unfinished_to() {
        let input = "
        SELECT CASE X
        CASE 1TO
        END SELECT";
        let result = parse_err_node(input);
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: end-of-statement"),
                Location::new(3, 15)
            )
        );
    }

    #[test]
    fn test_no_space_around_to() {
        let input = "
        SELECT CASE X
        CASE 1TO2
        END SELECT";
        let result = parse_err_node(input);
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: end-of-statement"),
                Location::new(3, 15)
            )
        );
    }

    #[test]
    fn test_no_space_after_to() {
        let input = "
        SELECT CASE X
        CASE 1 TO2
        END SELECT";
        let result = parse_err_node(input);
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: end-of-statement"),
                Location::new(3, 16)
            )
        );
    }

    #[test]
    fn test_no_space_before_to() {
        let input = "
        SELECT CASE X
        CASE 1TO 2
        END SELECT";
        let result = parse_err_node(input);
        assert_eq!(
            result,
            QErrorNode::Pos(
                QError::syntax_error("Expected: end-of-statement"),
                Location::new(3, 15)
            )
        );
    }
}
