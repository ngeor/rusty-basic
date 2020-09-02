use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::pc::common::{and, map_default_to_not_found, or_vec};
use crate::parser::pc::map::map;
use crate::parser::pc::ws::{is_eol, one_or_more_leading, zero_or_more_leading};
use crate::parser::pc::*;
use crate::parser::pc_specific::{csv_zero_or_more, in_parenthesis};
use crate::parser::types::*;
use std::io::BufRead;

// SubCall                  ::= SubCallNoArgs | SubCallArgsNoParenthesis | SubCallArgsParenthesis
// SubCallNoArgs            ::= BareName [eof | eol | ' | <ws+>: ]
// SubCallArgsNoParenthesis ::= BareName<ws+>ExpressionNodes
// SubCallArgsParenthesis   ::= BareName(ExpressionNodes)
pub fn sub_call<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        and(
            name::bare_name(),
            or_vec(vec![
                // e.g. PRINT("hello", "world")
                map_default_to_not_found(in_parenthesis(csv_zero_or_more(
                    expression::expression_node(),
                ))),
                // e.g. PRINT "hello", "world"
                zero_or_more_leading(map_default_to_not_found(csv_zero_or_more(
                    expression::expression_node(),
                ))),
                // prevent against e.g. A = "oops"
                one_or_more_leading(zero_args_assignment_and_label_guard(
                    statement_terminator_after_whitespace,
                )),
                // prevent against e.g. A: or A="oops"
                zero_args_assignment_and_label_guard(statement_terminator),
            ]),
        ),
        |(n, r)| Statement::SubCall(n, r),
    )
}

fn statement_terminator(ch: char) -> bool {
    ch == '\'' || is_eol(ch)
}

fn statement_terminator_after_whitespace(ch: char) -> bool {
    statement_terminator(ch) || ch == ':'
}

fn zero_args_assignment_and_label_guard<T: BufRead + 'static>(
    is_statement_terminator: fn(char) -> bool,
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ArgumentNodes, QError>> {
    Box::new(move |reader| {
        reader.read().and_then(|(reader, opt_res)| match opt_res {
            Some(ch) => {
                let res: Option<ArgumentNodes> = if is_statement_terminator(ch) {
                    // found statement terminator
                    Some(vec![])
                } else {
                    // found something else
                    None
                };
                Ok((reader.undo(ch), res))
            }
            None => {
                // EOF e.g. PRINT followed by EOF
                Ok((reader, Some(vec![])))
            }
        })
    })
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
