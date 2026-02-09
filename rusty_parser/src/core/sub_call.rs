use rusty_pc::and::KeepRightCombiner;
use rusty_pc::*;

use crate::core::expression::{csv_expressions_first_guarded, expression_pos_p, property};
use crate::error::ParserError;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::equal_sign_ws;
use crate::*;

// SubCall                  ::= SubCallNoArgs | SubCallArgsNoParenthesis | SubCallArgsParenthesis
// SubCallNoArgs            ::= BareName [eof | eol | ' | <ws+>: ]
// SubCallArgsNoParenthesis ::= BareName<ws+>Expressions
// SubCallArgsParenthesis   ::= BareName(Expressions)

pub fn sub_call_or_assignment_p() -> impl Parser<StringView, Output = Statement, Error = ParserError>
{
    // TODO review excessive use of clone and boxed
    name_and_opt_eq_sign().then_with_in_context(
        ctx_parser()
            .map(|(name_expr, has_equal_sign): (Expression, bool)| {
                if has_equal_sign {
                    // we have the equal sign, it's an assignment
                    expression_pos_p()
                        .or_expected("variable=expression")
                        .map(move |right_side_expr| {
                            Statement::assignment(name_expr.clone(), right_side_expr)
                        })
                        .boxed()
                } else if property::is_qualified(&name_expr) {
                    // a left-side qualified variable can only be assigned to,
                    // i.e. SUBs can't be qualified
                    err_supplier(|| ParserError::expected("=").to_fatal()).boxed()
                } else {
                    // it's a sub call
                    let (bare_name, opt_args) = expr_to_bare_name_args(name_expr);
                    match opt_args {
                        Some(args) => {
                            supplier(move || Statement::sub_call(bare_name.clone(), args.clone()))
                                .boxed()
                        }
                        _ => csv_expressions_first_guarded()
                            .or_default()
                            .map(move |args| Statement::sub_call(bare_name.clone(), args))
                            .boxed(),
                    }
                }
            })
            .flatten(),
        |x| x.clone(),
        KeepRightCombiner,
    )
}

fn name_and_opt_eq_sign()
-> impl Parser<StringView, Output = (Expression, bool), Error = ParserError> {
    property::parser()
        .map(|p| p.element)
        .and_tuple(equal_sign_ws().to_option().map(|opt| opt.is_some()))
}

/// Converts a name expression into a sub bare name and optionally sub arguments.
/// Sub arguments are only present for `Expression:FunctionCall` (i.e. when
/// the sub already has parenthesis). For other cases arguments are resolved later.
fn expr_to_bare_name_args(name_expr: Expression) -> (BareName, Option<Expressions>) {
    match name_expr {
        // A(1,2) or A$(1,2)
        Expression::FunctionCall(name, args) => {
            // this one is easy, convert it to a sub
            (name.demand_bare(), Some(args))
        }
        // A or A$ (might have arguments after space)
        Expression::Variable(name, _) => (name.demand_bare(), None),
        // only possible if A.B is a sub, if left_name_expr contains a Function, abort
        Expression::Property(_, _, _) => (fold_to_bare_name(name_expr), None),
        _ => panic!("Unexpected name expression"),
    }
}

fn fold_to_bare_name(expr: Expression) -> BareName {
    match expr {
        Expression::Variable(name, _) => name.demand_bare(),
        Expression::Property(boxed_left_side, name, _) => {
            let left_side_name = fold_to_bare_name(*boxed_left_side);
            let bare_name = name.demand_bare();
            Name::dot_concat(left_side_name, bare_name)
        }
        _ => panic!("Illegal sub name {:?}", expr),
    }
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::test_utils::*;
    use crate::{BuiltInSub, assert_sub_call, *};

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
            Statement::sub_call("Flint".into(), vec!["Hello, world!".as_lit_expr(1, 7)])
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
                GlobalStatement::Statement(Statement::built_in_sub_call(
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
                GlobalStatement::sub_declaration("Hello".as_bare_name(2, 21), vec![],),
                // Hello
                GlobalStatement::Statement(Statement::sub_call("Hello".into(), vec![])),
                // SUB Hello
                GlobalStatement::SubImplementation(SubImplementation {
                    name: "Hello".as_bare_name(4, 13),
                    params: vec![],
                    body: vec![
                        Statement::sub_call("ENVIRON".into(), vec!["FOO=BAR".as_lit_expr(5, 21)])
                            .at_rc(5, 13)
                    ],
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
                GlobalStatement::sub_declaration(
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
                GlobalStatement::Statement(Statement::sub_call(
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
                    body: vec![
                        Statement::sub_call(
                            "ENVIRON".into(),
                            vec![
                                Expression::BinaryExpression(
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
                                .at_rc(5, 30)
                            ]
                        )
                        .at_rc(5, 13)
                    ],
                    is_static: false
                })
            ]
        );
    }
}
