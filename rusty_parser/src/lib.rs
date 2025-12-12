mod assignment;
mod built_ins;
mod comment;
mod constant;
mod declaration;
mod def_type;
mod dim;
mod dim_name;
mod do_loop;
mod error;
mod exit;
mod expression;
mod for_loop;
mod global_statement;
mod go_sub;
mod if_block;
mod implementation;
mod name;
mod on_error;
mod param_name;
mod pc;
pub mod pc_ng;
mod pc_specific;
mod print;
mod resume;
mod select_case;
mod statement;
mod statement_separator;
mod statements;
mod sub_call;
mod types;
mod user_defined_type;
mod var_name;
mod while_wend;

#[cfg(test)]
pub mod test_utils;

pub use built_ins::{BuiltInFunction, BuiltInSub};
pub use error::*;
pub use types::*;

use rusty_common::AtPos;
use std::fs::File;

use crate::global_statement::program_parser_p;
use crate::pc::*;
use crate::pc_specific::{create_file_tokenizer, create_string_tokenizer};

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
    let mut reader = create_file_tokenizer(f).unwrap();
    program_parser(&mut reader)
}

pub fn program_parser<I: Tokenizer + 'static>(reader: &mut I) -> Result<Program, ParseErrorPos> {
    match program_parser_p().parse(reader) {
        Ok(opt_program) => Ok(opt_program),
        Err(err) => Err(err.at_pos(reader.position())),
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

fn parse_main_str(input: String) -> Result<Program, ParseErrorPos> {
    let mut reader = create_string_tokenizer(input);
    program_parser(&mut reader)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use crate::types::*;
    use crate::BuiltInSub;
    use rusty_common::*;

    #[test]
    fn test_parse_fixture_fib() {
        let program = parse_file_no_pos("FIB.BAS");
        assert_eq!(
            program,
            vec![
                // DECLARE FUNCTION Fib! (N!)
                GlobalStatement::FunctionDeclaration(
                    "Fib!".as_name(1, 18),
                    vec![Parameter::new(
                        "N".into(),
                        ParamType::BuiltIn(TypeQualifier::BangSingle, BuiltInStyle::Compact)
                    )
                    .at_rc(1, 24)],
                ),
                // PRINT "Enter the number of fibonacci to calculate"
                GlobalStatement::Statement(Statement::Print(Print::one(
                    "Enter the number of fibonacci to calculate".as_lit_expr(2, 7)
                ))),
                // INPUT N
                GlobalStatement::Statement(Statement::BuiltInSubCall(
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
                                    Statement::Assignment(
                                        Expression::var_unresolved("Fib"),
                                        "N".as_var_expr(10, 15)
                                    )
                                    .at_rc(10, 9)
                                ],
                            },
                            else_if_blocks: vec![],
                            else_block: Some(vec![
                                // ELSE Fib = Fib(N - 1) + Fib(N - 2)
                                Statement::Assignment(
                                    Expression::var_unresolved("Fib"),
                                    Expression::BinaryExpression(
                                        Operator::Plus,
                                        Box::new(
                                            Expression::func(
                                                "Fib",
                                                vec![Expression::BinaryExpression(
                                                    Operator::Minus,
                                                    Box::new("N".as_var_expr(12, 19)),
                                                    Box::new(1.as_lit_expr(12, 23)),
                                                    ExpressionType::Unresolved
                                                )
                                                .at_rc(12, 21)]
                                            )
                                            .at_rc(12, 15)
                                        ),
                                        Box::new(
                                            Expression::func(
                                                "Fib",
                                                vec![Expression::BinaryExpression(
                                                    Operator::Minus,
                                                    Box::new("N".as_var_expr(12, 32)),
                                                    Box::new(2.as_lit_expr(12, 36)),
                                                    ExpressionType::Unresolved
                                                )
                                                .at_rc(12, 34)]
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
