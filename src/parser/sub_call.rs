use crate::char_reader::*;
use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::types::*;
use std::io::BufRead;

// SubCall                  ::= SubCallNoArgs | SubCallArgsNoParenthesis | SubCallArgsParenthesis
// SubCallNoArgs            ::= BareName [eof | eol | ' | <ws+>: ]
// SubCallArgsNoParenthesis ::= BareName<ws+>ExpressionNodes
// SubCallArgsParenthesis   ::= BareName(ExpressionNodes)
pub fn sub_call<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    map_ng(
        and_ng(
            name::bare_name(),
            or_vec_ng(vec![
                in_parenthesis(csv_zero_or_more(expression::expression_node())),
                skipping_whitespace_ng(csv_zero_or_more(expression::expression_node())),
                map_ng(
                    and_ng(
                        read_any_whitespace(),
                        zero_args_assignment_and_label_guard(true),
                    ),
                    |(_, r)| r,
                ),
                zero_args_assignment_and_label_guard(false),
            ]),
        ),
        |(n, r)| Statement::SubCall(n, r),
    )
}

pub fn zero_args_assignment_and_label_guard<T: BufRead + 'static>(
    allow_colon: bool,
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ArgumentNodes, QErrorNode>)> {
    Box::new(move |reader| {
        let (reader, next) = reader.read();
        match next {
            Ok(ch) => {
                if ch == '\'' || ch == '\r' || ch == '\n' || (allow_colon && ch == ':') {
                    (reader.undo(ch), Ok(vec![]))
                } else {
                    reader.undo_and_err_not_found(ch)
                }
            }
            Err(err) => {
                if err.is_not_found_err() {
                    // EOF is ok
                    (reader, Ok(vec![]))
                } else {
                    (reader, Err(err))
                }
            }
        }
    })
}

#[deprecated]
pub fn take_if_sub_call<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    Box::new(in_transaction_pc(apply(
        |(l, opt_r)| l.map(|n| Statement::SubCall(n, opt_r.unwrap_or_default())),
        zip_allow_right_none(
            in_transaction_pc(switch(
                |(n, opt_blocker)| match opt_blocker {
                    Some(_) => None,
                    None => Some(n),
                },
                zip_allow_right_none(
                    name::take_if_bare_name_node(),
                    or(detect_label_and_abort(), detect_assignment_and_abort()),
                ),
            )),
            or(take_args_parenthesis(), take_args_no_parenthesis()),
        ),
    )))
}

#[deprecated]
fn detect_label_and_abort<T: BufRead + 'static>() -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<bool>> {
    apply(|_| true, take_if_symbol(':'))
}

#[deprecated]
fn detect_assignment_and_abort<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<bool>> {
    apply(|_| true, skipping_whitespace(take_if_symbol('=')))
}

#[deprecated]
fn take_args_no_parenthesis<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<Vec<ExpressionNode>>> {
    Box::new(with_leading_whitespace(csv(
        expression::take_if_expression_node(),
    )))
}

#[deprecated]
fn take_args_parenthesis<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<Vec<ExpressionNode>>> {
    apply(
        |(_, r)| r,
        between('(', ')', csv(expression::take_if_expression_node())),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{
        DeclaredName, Expression, Name, Operand, Statement, TopLevelToken, TypeQualifier,
    };

    #[test]
    fn test_parse_sub_call_no_args() {
        let input = "PRINT";
        let program = parse(input).demand_single_statement();
        assert_eq!(program, Statement::SubCall("PRINT".into(), vec![]));
    }

    #[test]
    fn test_parse_sub_call_single_arg_string_literal() {
        let input = "PRINT \"Hello, world!\"";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::SubCall("PRINT".into(), vec!["Hello, world!".as_lit_expr(1, 7)])
        );
    }

    #[test]
    fn test_parse_fixture_hello1() {
        let program = parse_file("HELLO1.BAS").demand_single_statement();
        assert_eq!(
            program,
            Statement::SubCall("PRINT".into(), vec!["Hello, world!".as_lit_expr(1, 7)])
        );
    }

    #[test]
    fn test_parse_fixture_hello2() {
        let program = parse_file("HELLO2.BAS").demand_single_statement();
        assert_eq!(
            program,
            Statement::SubCall(
                "PRINT".into(),
                vec!["Hello".as_lit_expr(1, 7), "world!".as_lit_expr(1, 16)]
            )
        );
    }

    #[test]
    fn test_parse_fixture_hello_system() {
        let program = parse_file("HELLO_S.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::SubCall(
                    "PRINT".into(),
                    vec!["Hello, world!".as_lit_expr(1, 7)]
                )),
                TopLevelToken::Statement(Statement::SubCall("SYSTEM".into(), vec![])),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_input() {
        let program = parse_file("INPUT.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::SubCall(
                    "INPUT".into(),
                    vec!["N".as_var_expr(1, 7)]
                )),
                TopLevelToken::Statement(Statement::SubCall(
                    "PRINT".into(),
                    vec!["N".as_var_expr(2, 7)]
                )),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_environ() {
        let program = parse_file("ENVIRON.BAS").strip_location();
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::SubCall(
                "PRINT".into(),
                vec![Expression::FunctionCall(
                    Name::from("ENVIRON$"),
                    vec!["PATH".as_lit_expr(1, 16)]
                )
                .at_rc(1, 7)]
            ))]
        );
    }

    #[test]
    fn test_parse_sub_call_user_defined_no_args() {
        let input = r#"
        DECLARE SUB Hello
        Hello
        SUB Hello
            ENVIRON "FOO=BAR"
        END SUB
        "#;
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                TopLevelToken::SubDeclaration("Hello".as_bare_name(2, 21), vec![],),
                // Hello
                TopLevelToken::Statement(Statement::SubCall("Hello".into(), vec![])),
                // SUB Hello
                TopLevelToken::SubImplementation(
                    "Hello".as_bare_name(4, 13),
                    vec![],
                    vec![
                        Statement::SubCall("ENVIRON".into(), vec!["FOO=BAR".as_lit_expr(5, 21)])
                            .at_rc(5, 13)
                    ],
                )
            ]
        );
    }

    #[test]
    fn test_parse_sub_call_user_defined_two_args() {
        let input = r#"
        DECLARE SUB Hello(N$, V$)
        Hello "FOO", "BAR"
        SUB Hello(N$, V$)
            ENVIRON N$ + "=" + V$
        END SUB
        "#;
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                TopLevelToken::SubDeclaration(
                    "Hello".as_bare_name(2, 21),
                    vec![
                        DeclaredName::compact("N", TypeQualifier::DollarString).at_rc(2, 27),
                        DeclaredName::compact("V", TypeQualifier::DollarString).at_rc(2, 31)
                    ],
                ),
                // Hello
                TopLevelToken::Statement(Statement::SubCall(
                    "Hello".into(),
                    vec!["FOO".as_lit_expr(3, 15), "BAR".as_lit_expr(3, 22)]
                )),
                // SUB Hello
                TopLevelToken::SubImplementation(
                    "Hello".as_bare_name(4, 13),
                    vec![
                        DeclaredName::compact("N", TypeQualifier::DollarString).at_rc(4, 19),
                        DeclaredName::compact("V", TypeQualifier::DollarString).at_rc(4, 23)
                    ],
                    vec![Statement::SubCall(
                        "ENVIRON".into(),
                        vec![Expression::BinaryExpression(
                            Operand::Plus,
                            Box::new("N$".as_var_expr(5, 21)),
                            Box::new(
                                Expression::BinaryExpression(
                                    Operand::Plus,
                                    Box::new("=".as_lit_expr(5, 26)),
                                    Box::new("V$".as_var_expr(5, 32))
                                )
                                .at_rc(5, 30)
                            )
                        )
                        .at_rc(5, 24)]
                    )
                    .at_rc(5, 13)],
                )
            ]
        );
    }

    #[test]
    fn test_close_file_handle() {
        let input = "CLOSE #1";
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::SubCall(
                "CLOSE".into(),
                vec![Expression::FileHandle(1.into()).at_rc(1, 7)]
            ))]
        );
    }

    #[test]
    fn test_inline_comment() {
        let input = "CLOSE #1 ' closes the file";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::SubCall(
                    "CLOSE".into(),
                    vec![Expression::FileHandle(1.into()).at_rc(1, 7)]
                ))
                .at_rc(1, 1),
                TopLevelToken::Statement(Statement::Comment(" closes the file".to_string(),))
                    .at_rc(1, 10)
            ]
        );
    }
}
