use crate::pc::*;
use crate::pc_specific::*;
use crate::types::*;
use crate::{expression, ParseError};
use rusty_common::*;

// SubCall                  ::= SubCallNoArgs | SubCallArgsNoParenthesis | SubCallArgsParenthesis
// SubCallNoArgs            ::= BareName [eof | eol | ' | <ws+>: ]
// SubCallArgsNoParenthesis ::= BareName<ws+>Expressions
// SubCallArgsParenthesis   ::= BareName(Expressions)

pub fn sub_call_or_assignment_p<I: Tokenizer + 'static>() -> impl Parser<I, Output = Statement> {
    SubCallOrAssignment
}

struct SubCallOrAssignment;

impl<I: Tokenizer + 'static> Parser<I> for SubCallOrAssignment {
    type Output = Statement;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        let (
            Positioned {
                element: name_expr, ..
            },
            opt_equal_sign,
        ) = match Self::name_and_opt_eq_sign().parse(tokenizer) {
            ParseResult::Ok(x) => x,
            ParseResult::None => return ParseResult::None,
            ParseResult::Expected(err) => return ParseResult::Expected(err),
            ParseResult::Err(err) => return ParseResult::Err(err),
        };

        match opt_equal_sign {
            Some(_) => expression::expression_pos_p()
                .or_syntax_error("Expected: expression for assignment")
                .parse(tokenizer)
                .map(|right_side_expr| Statement::Assignment(name_expr, right_side_expr)),
            _ => match expr_to_bare_name_args(name_expr) {
                Ok((bare_name, Some(args))) => ParseResult::Ok(Statement::SubCall(bare_name, args)),
                Ok((bare_name, None)) => expression::csv_expressions_first_guarded()
                    .or_default()
                    .parse(tokenizer)
                    .map(|args| Statement::SubCall(bare_name, args)),
                Err(err) => ParseResult::Err(err),
            },
        }
    }
}

impl SubCallOrAssignment {
    fn name_and_opt_eq_sign<I: Tokenizer + 'static>(
    ) -> impl Parser<I, Output = (ExpressionPos, Option<Token>)> {
        expression::property::parser().and_opt_tuple(equal_sign())
    }
}

/// Converts a name expression into a sub bare name and optionally sub arguments.
/// Sub arguments are only present for `Expression:FunctionCall` (i.e. when
/// the sub already has parenthesis). For other cases arguments are resolved later.
fn expr_to_bare_name_args(
    name_expr: Expression,
) -> Result<(BareName, Option<Expressions>), ParseError> {
    match name_expr {
        // A(1,2) or A$(1,2)
        Expression::FunctionCall(name, args) => {
            // this one is easy, convert it to a sub
            demand_unqualified(name).map(|bare_name| (bare_name, Some(args)))
        }
        // A or A$ (might have arguments after space)
        Expression::Variable(name, _) => {
            demand_unqualified(name).map(|bare_name| (bare_name, None))
        }
        // only possible if A.B is a sub, if left_name_expr contains a Function, abort
        Expression::Property(_, _, _) => {
            fold_to_bare_name(name_expr).map(|bare_name| (bare_name, None))
        }
        _ => panic!("Unexpected name expression"),
    }
}

fn demand_unqualified(name: Name) -> Result<BareName, ParseError> {
    match name {
        Name::Bare(bare_name) => Ok(bare_name),
        _ => Err(ParseError::syntax_error("Sub cannot be qualified")),
    }
}

fn fold_to_bare_name(expr: Expression) -> Result<BareName, ParseError> {
    match expr {
        Expression::Variable(Name::Bare(bare_name), _) => Ok(bare_name),
        Expression::Property(boxed_left_side, Name::Bare(bare_name), _) => {
            let left_side_name = fold_to_bare_name(*boxed_left_side)?;
            Ok(Name::dot_concat(left_side_name, bare_name))
        }
        _ => Err(ParseError::syntax_error("Illegal sub name")),
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_sub_call;
    use crate::test_utils::*;
    use crate::*;
    use rusty_common::*;

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
            Statement::Print(Print::one("Hello, world!".as_lit_expr(1, 7)))
        );
    }

    #[test]
    fn test_parse_fixture_hello2() {
        let program = parse_file("HELLO2.BAS").demand_single_statement();
        assert_eq!(
            program,
            Statement::Print(Print {
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
        let program = parse_file_no_pos("HELLO_S.BAS");
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::Print(Print::one(
                    "Hello, world!".as_lit_expr(1, 7)
                ))),
                GlobalStatement::Statement(Statement::System),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_input() {
        let program = parse_file_no_pos("INPUT.BAS");
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::BuiltInSubCall(
                    BuiltInSub::Input,
                    vec![
                        0.as_lit_expr(1, 1), // no file number
                        "N".as_var_expr(1, 7)
                    ]
                )),
                GlobalStatement::Statement(Statement::Print(Print::one("N".as_var_expr(2, 7)))),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_environ() {
        let program = parse_file_no_pos("ENVIRON.BAS");
        assert_eq!(
            program,
            vec![GlobalStatement::Statement(Statement::Print(Print::one(
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
        let program = parse_str_no_pos(input);
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                GlobalStatement::SubDeclaration("Hello".as_bare_name(2, 21), vec![],),
                // Hello
                GlobalStatement::Statement(Statement::SubCall("Hello".into(), vec![])),
                // SUB Hello
                GlobalStatement::SubImplementation(SubImplementation {
                    name: "Hello".as_bare_name(4, 13),
                    params: vec![],
                    body: vec![Statement::SubCall(
                        "ENVIRON".into(),
                        vec!["FOO=BAR".as_lit_expr(5, 21)]
                    )
                    .at_rc(5, 13)],
                    is_static: false
                })
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
        let program = parse_str_no_pos(input);
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                GlobalStatement::SubDeclaration(
                    "Hello".as_bare_name(2, 21),
                    vec![
                        Parameter::new(
                            "N".into(),
                            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                        )
                        .at_rc(2, 27),
                        Parameter::new(
                            "V".into(),
                            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                        )
                        .at_rc(2, 31)
                    ],
                ),
                // Hello
                GlobalStatement::Statement(Statement::SubCall(
                    "Hello".into(),
                    vec!["FOO".as_lit_expr(3, 15), "BAR".as_lit_expr(3, 22)]
                )),
                // SUB Hello
                GlobalStatement::SubImplementation(SubImplementation {
                    name: "Hello".as_bare_name(4, 13),
                    params: vec![
                        Parameter::new(
                            "N".into(),
                            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                        )
                        .at_rc(4, 19),
                        Parameter::new(
                            "V".into(),
                            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                        )
                        .at_rc(4, 23)
                    ],
                    body: vec![Statement::SubCall(
                        "ENVIRON".into(),
                        vec![Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new(
                                Expression::BinaryExpression(
                                    Operator::Plus,
                                    Box::new("N$".as_var_expr(5, 21)),
                                    Box::new("=".as_lit_expr(5, 26)),
                                    ExpressionType::Unresolved
                                )
                                .at_rc(5, 24)
                            ),
                            Box::new("V$".as_var_expr(5, 32)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(5, 30)]
                    )
                    .at_rc(5, 13)],
                    is_static: false
                })
            ]
        );
    }
}
