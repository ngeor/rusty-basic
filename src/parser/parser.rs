use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::pc_specific::with_err_at;
use crate::parser::top_level_token;
use crate::parser::types::*;
use std::fs::File;

pub fn parse_main_file(f: File) -> Result<ProgramNode, QErrorNode> {
    let reader = EolReader::from(f);
    let (_, result) = with_err_at(top_level_token::top_level_tokens())(reader)?;
    Ok(result.unwrap_or_default())
}

#[cfg(test)]
pub fn parse_main_str<T: AsRef<[u8]> + 'static>(s: T) -> Result<ProgramNode, QErrorNode> {
    let reader = EolReader::from(s);
    let (_, result) = with_err_at(top_level_token::top_level_tokens())(reader)?;
    Ok(result.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::types::*;

    #[test]
    fn test_parse_fixture_fib() {
        let program = parse_file("FIB.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                // DECLARE FUNCTION Fib! (N!)
                TopLevelToken::FunctionDeclaration(
                    "Fib!".as_name(1, 18),
                    vec![
                        ParamName::new("N".into(), ParamType::Compact(TypeQualifier::BangSingle))
                            .at_rc(1, 24)
                    ],
                ),
                // PRINT "Enter the number of fibonacci to calculate"
                TopLevelToken::Statement(Statement::Print(PrintNode::one(
                    "Enter the number of fibonacci to calculate".as_lit_expr(2, 7)
                ))),
                // INPUT N
                TopLevelToken::Statement(Statement::SubCall(
                    BareName::from("INPUT"),
                    vec![
                        0.as_lit_expr(1, 1), // no file number
                        "N".as_var_expr(3, 7)
                    ]
                )),
                // FOR I = 0 TO N
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: Expression::var("I").at_rc(4, 5),
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
                TopLevelToken::FunctionImplementation(
                    Name::from("Fib").at_rc(8, 10),
                    vec![ParamName::new("N".into(), ParamType::Bare).at_rc(8, 15)],
                    vec![
                        // IF N <= 1 THEN
                        Statement::IfBlock(IfBlockNode {
                            if_block: ConditionalBlockNode {
                                // N <= 1
                                condition: Expression::BinaryExpression(
                                    Operator::LessOrEqual,
                                    Box::new("N".as_var_expr(9, 8)),
                                    Box::new(1.as_lit_expr(9, 13))
                                )
                                .at_rc(9, 10),
                                statements: vec![
                                    // Fib = N
                                    Statement::Assignment(
                                        Expression::var("Fib"),
                                        "N".as_var_expr(10, 15)
                                    )
                                    .at_rc(10, 9)
                                ],
                            },
                            else_if_blocks: vec![],
                            else_block: Some(vec![
                                // ELSE Fib = Fib(N - 1) + Fib(N - 2)
                                Statement::Assignment(
                                    Expression::var("Fib"),
                                    Expression::BinaryExpression(
                                        Operator::Plus,
                                        Box::new(
                                            Expression::func(
                                                "Fib",
                                                vec![Expression::BinaryExpression(
                                                    Operator::Minus,
                                                    Box::new("N".as_var_expr(12, 19)),
                                                    Box::new(1.as_lit_expr(12, 23)),
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
                                                )
                                                .at_rc(12, 34)]
                                            )
                                            .at_rc(12, 28)
                                        )
                                    )
                                    .at_rc(12, 26)
                                )
                                .at_rc(12, 9)
                            ])
                        })
                        .at_rc(9, 5)
                    ],
                ),
            ],
        );
    }
}
