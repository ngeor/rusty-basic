use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::pc::common::{and, drop_left, many, map_default_to_not_found, opt_seq3, or_vec};
use crate::parser::pc::map::{and_then, map};
use crate::parser::pc::ws::{is_eol, one_or_more_leading, zero_or_more_leading};
use crate::parser::pc::*;
use crate::parser::pc_specific::{csv_zero_or_more, in_parenthesis};
use crate::parser::types::*;
use std::io::BufRead;

// SubCall                  ::= SubCallNoArgs | SubCallArgsNoParenthesis | SubCallArgsParenthesis
// SubCallNoArgs            ::= BareName [eof | eol | ' | <ws+>: ]
// SubCallArgsNoParenthesis ::= BareName<ws+>ExpressionNodes
// SubCallArgsParenthesis   ::= BareName(ExpressionNodes)
pub fn sub_call_or_assignment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    and_then(
        and(
            name::name(),
            or_vec(vec![
                // e.g. PRINT("hello", "world")
                opt_seq3(
                    map_default_to_not_found(in_parenthesis(csv_zero_or_more(
                        expression::expression_node(),
                    ))),
                    map_default_to_not_found(many(drop_left(and(
                        read('.'),
                        name::any_word_without_dot(),
                    )))),
                    drop_left(and(
                        ws::zero_or_more_around(read('=')),
                        expression::expression_node(),
                    )),
                ),
                // e.g. PRINT "hello", "world"
                map(
                    zero_or_more_leading(map_default_to_not_found(csv_zero_or_more(
                        expression::expression_node(),
                    ))),
                    |x| (x, None, None),
                ),
                // assignment
                map(
                    drop_left(and(
                        ws::zero_or_more_around(read('=')),
                        expression::expression_node(),
                    )),
                    |x| (vec![], None, Some(x)),
                ),
                // prevent against e.g. A = "oops"
                map(
                    one_or_more_leading(zero_args_assignment_and_label_guard(
                        statement_terminator_after_whitespace,
                    )),
                    |x| (x, None, None),
                ),
                // prevent against e.g. A: or A="oops"
                map(
                    zero_args_assignment_and_label_guard(statement_terminator),
                    |x| (x, None, None),
                ),
            ]),
        ),
        |(n, (args, opt_elements, opt_assignment_r_value))| {
            match opt_assignment_r_value {
                Some(assignment_r_value) => {
                    // assignment
                    let mut name_expr: Expression = if args.is_empty() {
                        Expression::VariableName(n)
                    } else {
                        Expression::FunctionCall(n, args)
                    };
                    if let Some(elements) = opt_elements {
                        for element in elements {
                            name_expr =
                                Expression::Property(Box::new(name_expr), Name::Bare(element));
                        }
                    }
                    Ok(Statement::Assignment(name_expr, assignment_r_value))
                }
                None => {
                    if opt_elements.is_some() {
                        Err(QError::syntax_error("Sub cannot have properties"))
                    } else {
                        // sub call
                        match n {
                            Name::Bare(bare_name) => Ok(Statement::SubCall(bare_name, args)),
                            Name::Qualified(_) => {
                                Err(QError::syntax_error("Sub name cannot be qualified"))
                            }
                        }
                    }
                }
            }
        },
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
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNodes, QError>> {
    Box::new(move |reader| {
        reader.read().and_then(|(reader, opt_res)| match opt_res {
            Some(ch) => {
                let res: Option<ExpressionNodes> = if is_statement_terminator(ch) {
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
