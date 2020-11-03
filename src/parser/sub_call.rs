use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::pc::common::{drop_left, many, many_looking_back, seq2};
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc_specific::csv_zero_or_more;
use crate::parser::types::*;
use std::io::BufRead;

// SubCall                  ::= SubCallNoArgs | SubCallArgsNoParenthesis | SubCallArgsParenthesis
// SubCallNoArgs            ::= BareName [eof | eol | ' | <ws+>: ]
// SubCallArgsNoParenthesis ::= BareName<ws+>ExpressionNodes
// SubCallArgsParenthesis   ::= BareName(ExpressionNodes)
pub fn sub_call_or_assignment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    Box::new(move |r| {
        match expression::word::word()(r) {
            Ok((r, Some(name_expr))) => {
                // is there an equal sign following?
                match ws::zero_or_more_around(read('='))(r) {
                    Ok((r, Some(_))) => assignment(r, name_expr),
                    Ok((r, None)) => sub_call(r, name_expr),
                    Err((r, err)) => Err((r, err)),
                }
            }
            Ok((r, None)) => Ok((r, None)),
            Err((r, err)) => Err((r, err)),
        }
    })
}

fn assignment<T: BufRead + 'static>(
    r: EolReader<T>,
    name_expr: Expression,
) -> ReaderResult<EolReader<T>, Statement, QError> {
    match expression::demand_expression_node()(r) {
        Ok((r, Some(right_expr_node))) => {
            Ok((r, Some(Statement::Assignment(name_expr, right_expr_node))))
        }
        Ok((r, None)) => panic!("Got None from demand_expression_node, should not happen"),
        Err((r, err)) => Err((r, err)),
    }
}

fn sub_call<T: BufRead + 'static>(
    r: EolReader<T>,
    name_expr: Expression,
) -> ReaderResult<EolReader<T>, Statement, QError> {
    match name_expr {
        // A(1, 2) or A$(1, 2)
        Expression::FunctionCall(name, args) => {
            match name {
                Name::Bare(bare_name) => {
                    // this one is easy, convert it to a sub
                    Ok((r, Some(Statement::SubCall(bare_name, args))))
                }
                _ => Err((r, QError::syntax_error("Sub cannot be qualified"))),
            }
        }
        Expression::VariableName(name) => {
            // A or A$ (might have arguments after space)
            match name {
                Name::Bare(bare_name) => {
                    // are there any args after the space?
                    match sub_call_args_after_space()(r) {
                        Ok((r, opt_args)) => Ok((
                            r,
                            Some(Statement::SubCall(bare_name, opt_args.unwrap_or_default())),
                        )),
                        Err((r, err)) => Err((r, err)),
                    }
                }
                _ => Err((r, QError::syntax_error("Sub cannot be qualified"))),
            }
        }
        Expression::Property(left_name_expr, property_name) => {
            // only possible if A.B is a sub, if left_name_expr contains a Function, abort
            todo!()
        }
        _ => panic!("Unexpected name expression"),
    }
}

fn sub_call_args_after_space<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNodes, QError>> {
    map(
        seq2(
            // first expression after sub name
            expression::guarded_expression_node(),
            many(drop_left(seq2(
                // read comma after previous expression
                ws::zero_or_more_around(read(',')),
                // must have expression after comma
                expression::demand_expression_node(),
            ))),
        ),
        |(first_expr, mut remaining_expr)| {
            remaining_expr.insert(0, first_expr);
            remaining_expr
        },
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::assert_sub_call;
    use crate::common::*;
    use crate::parser::{
        Expression, Operator, ParamName, ParamType, PrintArg, PrintNode, Statement, TopLevelToken,
        TypeQualifier,
    };

    #[test]
    fn test_parse_sub_call_no_args() {
        let input = "Flint";
        let program = parse(input).demand_single_statement();
        assert_sub_call!(program, "Flint");
    }

    #[test]
    fn test_parse_sub_call_single_arg_string_literal() {
        let input = "Flint \"Hello, world!\"";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::SubCall("Flint".into(), vec!["Hello, world!".as_lit_expr(1, 7)])
        );
    }

    #[test]
    fn test_parse_fixture_hello1() {
        let program = parse_file("HELLO1.BAS").demand_single_statement();
        assert_eq!(
            program,
            Statement::Print(PrintNode::one("Hello, world!".as_lit_expr(1, 7)))
        );
    }

    #[test]
    fn test_parse_fixture_hello2() {
        let program = parse_file("HELLO2.BAS").demand_single_statement();
        assert_eq!(
            program,
            Statement::Print(PrintNode {
                file_number: None,
                lpt1: false,
                format_string: None,
                args: vec![
                    PrintArg::Expression("Hello".as_lit_expr(1, 7)),
                    PrintArg::Comma,
                    PrintArg::Expression("world!".as_lit_expr(1, 16))
                ]
            })
        );
    }

    #[test]
    fn test_parse_fixture_hello_system() {
        let program = parse_file("HELLO_S.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Print(PrintNode::one(
                    "Hello, world!".as_lit_expr(1, 7)
                ))),
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
                    vec![
                        0.as_lit_expr(1, 1), // no file number
                        "N".as_var_expr(1, 7)
                    ]
                )),
                TopLevelToken::Statement(Statement::Print(PrintNode::one("N".as_var_expr(2, 7)))),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_environ() {
        let program = parse_file("ENVIRON.BAS").strip_location();
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::Print(PrintNode::one(
                Expression::func("ENVIRON$", vec!["PATH".as_lit_expr(1, 16)]).at_rc(1, 7)
            )))]
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
                        ParamName::new("N".into(), ParamType::Compact(TypeQualifier::DollarString))
                            .at_rc(2, 27),
                        ParamName::new("V".into(), ParamType::Compact(TypeQualifier::DollarString))
                            .at_rc(2, 31)
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
                        ParamName::new("N".into(), ParamType::Compact(TypeQualifier::DollarString))
                            .at_rc(4, 19),
                        ParamName::new("V".into(), ParamType::Compact(TypeQualifier::DollarString))
                            .at_rc(4, 23)
                    ],
                    vec![Statement::SubCall(
                        "ENVIRON".into(),
                        vec![Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("N$".as_var_expr(5, 21)),
                            Box::new(
                                Expression::BinaryExpression(
                                    Operator::Plus,
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
        let program = parse(input).demand_single_statement();
        assert_sub_call!(program, "CLOSE", Expression::IntegerLiteral(1));
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
                    vec![1.as_lit_expr(1, 7)]
                ))
                .at_rc(1, 1),
                TopLevelToken::Statement(Statement::Comment(" closes the file".to_string(),))
                    .at_rc(1, 10)
            ]
        );
    }
}
