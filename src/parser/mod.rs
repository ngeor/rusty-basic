use std::fs::File;

use crate::common::*;
use crate::parser::base::parsers::Parser;
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::specific::create_file_tokenizer;
use crate::parser::top_level_token::TopLevelTokensParser;

pub use self::types::*;

mod assignment;
pub mod base;
mod comment;
mod constant;
mod declaration;
mod def_type;
mod dim;
mod dim_name;
mod do_loop;
mod exit;
pub mod expression;
mod for_loop;
mod go_sub;
mod if_block;
mod implementation;
pub mod name;
mod on_error;
mod param_name;
mod print;
mod resume;
mod select_case;
pub mod specific;
pub mod statement;
mod statement_separator;
pub mod statements;
pub mod sub_call;
mod top_level_token;
mod type_qualifier;
mod types;
mod user_defined_type;
mod while_wend;

#[cfg(test)]
pub mod test_utils;

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
pub fn parse_main_file(f: File) -> Result<ProgramNode, QErrorNode> {
    let mut reader = create_file_tokenizer(f);
    parse_reader(&mut reader)
}

#[cfg(test)]
use crate::parser::specific::create_string_tokenizer;
#[cfg(test)]
pub fn parse_main_str<T: AsRef<[u8]> + 'static>(s: T) -> Result<ProgramNode, QErrorNode> {
    let mut reader = create_string_tokenizer(s);
    parse_reader(&mut reader)
}

fn parse_reader(reader: &mut impl Tokenizer) -> Result<ProgramNode, QErrorNode> {
    match TopLevelTokensParser::new().parse(reader) {
        Ok(opt_program) => Ok(opt_program.unwrap_or_default()),
        Err(err) => Err(ErrorEnvelope::Pos(err, reader.position().into())),
    }
}

#[cfg(test)]
mod tests {
    use crate::built_ins::BuiltInSub;
    use crate::common::*;
    use crate::parser::types::*;

    use super::test_utils::*;

    #[test]
    fn test_parse_fixture_fib() {
        let program = parse_file("FIB.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                // DECLARE FUNCTION Fib! (N!)
                TopLevelToken::FunctionDeclaration(
                    "Fib!".as_name(1, 18),
                    vec![ParamName::new(
                        "N".into(),
                        ParamType::BuiltIn(TypeQualifier::BangSingle, BuiltInStyle::Compact)
                    )
                    .at_rc(1, 24)],
                ),
                // PRINT "Enter the number of fibonacci to calculate"
                TopLevelToken::Statement(Statement::Print(PrintNode::one(
                    "Enter the number of fibonacci to calculate".as_lit_expr(2, 7)
                ))),
                // INPUT N
                TopLevelToken::Statement(Statement::BuiltInSubCall(
                    BuiltInSub::Input,
                    vec![
                        0.as_lit_expr(1, 1), // no file number
                        "N".as_var_expr(3, 7)
                    ]
                )),
                // FOR I = 0 TO N
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: Expression::var_unresolved("I").at_rc(4, 5),
                    lower_bound: 0.as_lit_expr(4, 9),
                    upper_bound: "N".as_var_expr(4, 14),
                    step: None,
                    statements: vec![
                        // PRINT "Fibonacci of ", I, " is ", Fib(I)
                        Statement::Print(PrintNode {
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
                TopLevelToken::FunctionImplementation(FunctionImplementation {
                    name: Name::from("Fib").at_rc(8, 10),
                    params: vec![ParamName::new("N".into(), ParamType::Bare).at_rc(8, 15)],
                    body: vec![
                        // IF N <= 1 THEN
                        Statement::IfBlock(IfBlockNode {
                            if_block: ConditionalBlockNode {
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
