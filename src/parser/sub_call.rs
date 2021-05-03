use crate::common::*;
use crate::parser::expression;
use crate::parser::pc::*;
use crate::parser::types::*;

// SubCall                  ::= SubCallNoArgs | SubCallArgsNoParenthesis | SubCallArgsParenthesis
// SubCallNoArgs            ::= BareName [eof | eol | ' | <ws+>: ]
// SubCallArgsNoParenthesis ::= BareName<ws+>ExpressionNodes
// SubCallArgsParenthesis   ::= BareName(ExpressionNodes)

pub fn sub_call_or_assignment_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    SubCallOrAssignment {}
}

pub struct SubCallOrAssignment {}

impl<R> Parser<R> for SubCallOrAssignment
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    type Output = Statement;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = Self::name_and_opt_eq_sign().parse(reader)?;
        match opt_item {
            Some((name_expr, opt_equal_sign)) => match opt_equal_sign {
                Some(_) => {
                    let (reader, opt_v) =
                        expression::demand_expression_node_p("Expected: expression for assignment")
                            .parse(reader)?;
                    Ok((
                        reader,
                        Some(Statement::Assignment(name_expr, opt_v.unwrap())),
                    ))
                }
                _ => match expr_to_bare_name_args(name_expr) {
                    Ok((bare_name, Some(args))) => {
                        Ok((reader, Some(Statement::SubCall(bare_name, args))))
                    }
                    Ok((bare_name, None)) => {
                        let (reader, args) = expression::expression_nodes_p().parse(reader)?;
                        Ok((
                            reader,
                            Some(Statement::SubCall(bare_name, args.unwrap_or_default())),
                        ))
                    }
                    Err(err) => Err((reader, err)),
                },
            },
            _ => Ok((reader, None)),
        }
    }
}

impl SubCallOrAssignment {
    fn name_and_opt_eq_sign<R>() -> impl Parser<R, Output = (Expression, Option<char>)>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        expression::word::word_p().and_opt(item_p('=').surrounded_by_opt_ws())
    }
}

/// Converts a name expression into a sub bare name and optionally sub arguments.
/// Sub arguments are only present for `Expression:FunctionCall` (i.e. when
/// the sub already has parenthesis). For other cases arguments are resolved later.
fn expr_to_bare_name_args(
    name_expr: Expression,
) -> Result<(BareName, Option<ExpressionNodes>), QError> {
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

fn demand_unqualified(name: Name) -> Result<BareName, QError> {
    match name {
        Name::Bare(bare_name) => Ok(bare_name),
        _ => Err(QError::syntax_error("Sub cannot be qualified")),
    }
}

fn fold_to_bare_name(expr: Expression) -> Result<BareName, QError> {
    match expr {
        Expression::Variable(Name::Bare(bare_name), _) => Ok(bare_name),
        Expression::Property(boxed_left_side, Name::Bare(bare_name), _) => {
            let left_side_name = fold_to_bare_name(*boxed_left_side)?;
            Ok(left_side_name + '.' + bare_name)
        }
        _ => Err(QError::syntax_error("Illegal sub name")),
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_sub_call;
    use crate::common::*;
    use crate::parser::{
        BuiltInStyle, Expression, ExpressionType, Operator, ParamName, ParamType, PrintArg,
        PrintNode, Statement, SubImplementation, TopLevelToken, TypeQualifier,
    };

    use super::super::test_utils::*;
    use crate::built_ins::BuiltInSub;

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
                TopLevelToken::Statement(Statement::System),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_input() {
        let program = parse_file("INPUT.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::BuiltInSubCall(
                    BuiltInSub::Input,
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
                TopLevelToken::SubImplementation(SubImplementation {
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
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                TopLevelToken::SubDeclaration(
                    "Hello".as_bare_name(2, 21),
                    vec![
                        ParamName::new(
                            "N".into(),
                            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                        )
                        .at_rc(2, 27),
                        ParamName::new(
                            "V".into(),
                            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                        )
                        .at_rc(2, 31)
                    ],
                ),
                // Hello
                TopLevelToken::Statement(Statement::SubCall(
                    "Hello".into(),
                    vec!["FOO".as_lit_expr(3, 15), "BAR".as_lit_expr(3, 22)]
                )),
                // SUB Hello
                TopLevelToken::SubImplementation(SubImplementation {
                    name: "Hello".as_bare_name(4, 13),
                    params: vec![
                        ParamName::new(
                            "N".into(),
                            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                        )
                        .at_rc(4, 19),
                        ParamName::new(
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
