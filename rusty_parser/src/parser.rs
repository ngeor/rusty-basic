use std::fs::File;

use rusty_common::AtPos;
use rusty_pc::*;

use crate::error::ParseErrorPos;
use crate::input::RcStringView;
use crate::specific::{Program, create_file_tokenizer, create_string_tokenizer, program_parser_p};

/// Parses a QBasic file.
///
/// Syntax reference
///
/// ```txt
/// (* zero or more whitespace *)
/// <opt-ws> ::= "" | <ws>
/// (* at least one whitespace *)
/// <ws> ::= " " | " "<ws>
/// <letter> ::= "A".."Z" | "a".."z"
/// <digit> ::= "0".."9"
/// ```
pub fn parse_main_file(f: File) -> Result<Program, ParseErrorPos> {
    let reader = create_file_tokenizer(f).unwrap();
    program_parser(reader)
}

pub fn program_parser(reader: RcStringView) -> Result<Program, ParseErrorPos> {
    match program_parser_p().parse(reader) {
        Ok((_, program)) => Ok(program),
        Err((_, input, err)) => Err(err.at_pos(input.position())),
    }
}

/// Parses the given string and demands success.
///
/// # Panics
///
/// If the parser has an error.
pub fn parse<S>(input: S) -> Program
where
    S: AsRef<str>,
{
    let s = input.as_ref().to_string();
    parse_main_str(s).expect("Could not parse program")
}

pub fn parse_main_str(input: String) -> Result<Program, ParseErrorPos> {
    let reader = create_string_tokenizer(input);
    program_parser(reader)
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::BuiltInSub;
    use crate::specific::*;
    use crate::test_utils::*;

    #[test]
    fn test_parse_fixture_fib() {
        let program = parse_file_no_pos("FIB.BAS");
        assert_eq!(
            program,
            vec![
                // DECLARE FUNCTION Fib! (N!)
                GlobalStatement::function_declaration(
                    "Fib!".as_name(1, 18),
                    vec![
                        Parameter::new(
                            "N".into(),
                            ParamType::BuiltIn(TypeQualifier::BangSingle, BuiltInStyle::Compact)
                        )
                        .at_rc(1, 24)
                    ],
                ),
                // PRINT "Enter the number of fibonacci to calculate"
                GlobalStatement::Statement(Statement::Print(Print::one(
                    "Enter the number of fibonacci to calculate".as_lit_expr(2, 7)
                ))),
                // INPUT N
                GlobalStatement::Statement(Statement::built_in_sub_call(
                    BuiltInSub::Input,
                    vec![
                        0.as_lit_expr(1, 1), // no file number
                        "N".as_var_expr(3, 7)
                    ]
                )),
                // FOR I = 0 TO N
                GlobalStatement::Statement(Statement::ForLoop(ForLoop {
                    variable_name: Expression::var_unresolved("I").at_rc(4, 5),
                    lower_bound: 0.as_lit_expr(4, 9),
                    upper_bound: "N".as_var_expr(4, 14),
                    step: None,
                    statements: vec![
                        // PRINT "Fibonacci of ", I, " is ", Fib(I)
                        Statement::Print(Print {
                            file_number: None,
                            lpt1: false,
                            format_string: None,
                            args: vec![
                                PrintArg::Expression("Fibonacci of".as_lit_expr(5, 11)),
                                PrintArg::Comma,
                                PrintArg::Expression("I".as_var_expr(5, 27)),
                                PrintArg::Comma,
                                PrintArg::Expression("is".as_lit_expr(5, 30)),
                                PrintArg::Comma,
                                PrintArg::Expression(
                                    Expression::func("Fib", vec!["I".as_var_expr(5, 40)],)
                                        .at_rc(5, 36)
                                ),
                            ]
                        })
                        .at_rc(5, 5),
                    ],
                    next_counter: None,
                })),
                // FUNCTION Fib (N)
                GlobalStatement::FunctionImplementation(FunctionImplementation {
                    name: Name::from("Fib").at_rc(8, 10),
                    params: vec![Parameter::new("N".into(), ParamType::Bare).at_rc(8, 15)],
                    body: vec![
                        // IF N <= 1 THEN
                        Statement::IfBlock(IfBlock {
                            if_block: ConditionalBlock {
                                // N <= 1
                                condition: Expression::BinaryExpression(
                                    Operator::LessOrEqual,
                                    Box::new("N".as_var_expr(9, 8)),
                                    Box::new(1.as_lit_expr(9, 13)),
                                    ExpressionType::Unresolved
                                )
                                .at_rc(9, 10),
                                statements: vec![
                                    // Fib = N
                                    Statement::assignment(
                                        Expression::var_unresolved("Fib"),
                                        "N".as_var_expr(10, 15)
                                    )
                                    .at_rc(10, 9)
                                ],
                            },
                            else_if_blocks: vec![],
                            else_block: Some(vec![
                                // ELSE Fib = Fib(N - 1) + Fib(N - 2)
                                Statement::assignment(
                                    Expression::var_unresolved("Fib"),
                                    Expression::BinaryExpression(
                                        Operator::Plus,
                                        Box::new(
                                            Expression::func(
                                                "Fib",
                                                vec![
                                                    Expression::BinaryExpression(
                                                        Operator::Minus,
                                                        Box::new("N".as_var_expr(12, 19)),
                                                        Box::new(1.as_lit_expr(12, 23)),
                                                        ExpressionType::Unresolved
                                                    )
                                                    .at_rc(12, 21)
                                                ]
                                            )
                                            .at_rc(12, 15)
                                        ),
                                        Box::new(
                                            Expression::func(
                                                "Fib",
                                                vec![
                                                    Expression::BinaryExpression(
                                                        Operator::Minus,
                                                        Box::new("N".as_var_expr(12, 32)),
                                                        Box::new(2.as_lit_expr(12, 36)),
                                                        ExpressionType::Unresolved
                                                    )
                                                    .at_rc(12, 34)
                                                ]
                                            )
                                            .at_rc(12, 28)
                                        ),
                                        ExpressionType::Unresolved
                                    )
                                    .at_rc(12, 26)
                                )
                                .at_rc(12, 9)
                            ])
                        })
                        .at_rc(9, 5)
                    ],
                    is_static: false
                }),
            ],
        );
    }
}
