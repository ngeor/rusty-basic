use crate::lexer::{BufLexer, LexemeNode, LexerError};
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

mod assignment;
mod declaration;
mod def_type;
mod expression;
mod for_loop;
mod function_implementation;
mod if_block;
mod name;
mod parse_result;
mod statement;
mod sub_call;
mod types;

#[cfg(test)]
mod test_utils;

pub use self::expression::*;
pub use self::for_loop::*;
pub use self::if_block::*;
pub use self::name::*;
pub use self::statement::*;
pub use self::types::*;

use parse_result::ParseResult;

#[derive(Debug)]
pub struct Parser<T: BufRead> {
    pub buf_lexer: BufLexer<T>,
}

impl<T: BufRead> Parser<T> {
    pub fn new(buf_lexer: BufLexer<T>) -> Parser<T> {
        Parser { buf_lexer }
    }

    pub fn parse(&mut self) -> Result<ProgramNode, LexerError> {
        let mut v: Vec<TopLevelTokenNode> = vec![];
        loop {
            self.buf_lexer.skip_whitespace_and_eol()?;
            let x = self._parse_top_level_token()?;
            match x {
                Some(t) => v.push(t),
                _ => break,
            };
        }
        Ok(v)
    }

    fn _parse_top_level_token(&mut self) -> Result<Option<TopLevelTokenNode>, LexerError> {
        if let Some(d) = self.try_parse_declaration()? {
            Ok(Some(d))
        } else if let Some(f) = self.try_parse_function_implementation()? {
            Ok(Some(f))
        } else if let Some(x) = self.try_parse_def_type()? {
            Ok(Some(x))
        } else if let Some(s) = self._try_parse_statement_as_top_level_token()? {
            Ok(Some(s))
        } else {
            let lexeme = self.buf_lexer.read()?;
            match lexeme {
                LexemeNode::EOF(_) => {
                    self.buf_lexer.consume();
                    Ok(None)
                }
                _ => Err(LexerError::Unexpected(
                    format!("Unexpected top-level token"),
                    lexeme,
                )),
            }
        }
    }

    fn _try_parse_statement_as_top_level_token(
        &mut self,
    ) -> Result<Option<TopLevelTokenNode>, LexerError> {
        match self.try_parse_statement()? {
            ParseResult::Match(s) => Ok(Some(TopLevelTokenNode::Statement(s))),
            _ => Ok(None),
        }
    }
}

// bytes || &str -> Parser
impl<T> From<T> for Parser<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        Parser::new(BufLexer::from(input))
    }
}

// File -> Parser
impl From<File> for Parser<BufReader<File>> {
    fn from(input: File) -> Self {
        Parser::new(BufLexer::from(input))
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;
    use crate::common::Location;

    #[test]
    fn test_parse_fixture_fib() {
        let program = parse_file("FIB.BAS");
        assert_eq!(
            program,
            vec![
                // DECLARE FUNCTION Fib! (N!)
                TopLevelTokenNode::FunctionDeclaration(FunctionDeclarationNode::new(
                    "Fib!".as_name(1, 18),
                    vec!["N!".as_name(1, 24)],
                    Location::new(1, 1)
                )),
                // PRINT "Enter the number of fibonacci to calculate"
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "PRINT".as_name(2, 1),
                    vec!["Enter the number of fibonacci to calculate".as_lit_expr(2, 7)],
                )),
                // INPUT N
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "INPUT".as_name(3, 1),
                    vec!["N".as_var_expr(3, 7)]
                )),
                // FOR I = 0 TO N
                TopLevelTokenNode::Statement(StatementNode::ForLoop(ForLoopNode {
                    variable_name: "I".as_name(4, 5),
                    lower_bound: 0.as_lit_expr(4, 9),
                    upper_bound: "N".as_var_expr(4, 14),
                    step: None,
                    statements: vec![
                        // PRINT "Fibonacci of ", I, " is ", Fib(I)
                        StatementNode::SubCall(
                            "PRINT".as_name(5, 5),
                            vec![
                                "Fibonacci of".as_lit_expr(5, 11),
                                "I".as_var_expr(5, 27),
                                "is".as_lit_expr(5, 30),
                                ExpressionNode::FunctionCall(
                                    "Fib".as_name(5, 36),
                                    vec!["I".as_var_expr(5, 40)],
                                ),
                            ],
                        ),
                    ],
                    next_counter: None,
                    pos: Location::new(4, 1)
                })),
                // FUNCTION Fib (N)
                TopLevelTokenNode::FunctionImplementation(FunctionImplementationNode {
                    name: "Fib".as_name(8, 10),
                    parameters: vec!["N".as_name(8, 15)],
                    block: vec![
                        // IF N <= 1 THEN
                        StatementNode::IfBlock(IfBlockNode {
                            if_block: ConditionalBlockNode {
                                // N <= 1
                                condition: ExpressionNode::BinaryExpression(
                                    OperandNode::new(
                                        Operand::LessOrEqualThan,
                                        Location::new(9, 10)
                                    ),
                                    Box::new("N".as_var_expr(9, 8)),
                                    Box::new(1.as_lit_expr(9, 13))
                                ),
                                block: vec![
                                    // Fib = N
                                    StatementNode::Assignment(
                                        "Fib".as_name(10, 9),
                                        "N".as_var_expr(10, 15)
                                    )
                                ],
                                pos: Location::new(9, 5)
                            },
                            else_if_blocks: vec![],
                            else_block: Some(vec![
                                // ELSE Fib = Fib(N - 1) + Fib(N - 2)
                                StatementNode::Assignment(
                                    "Fib".as_name(12, 9),
                                    ExpressionNode::BinaryExpression(
                                        OperandNode::new(Operand::Plus, Location::new(12, 26)),
                                        Box::new(ExpressionNode::FunctionCall(
                                            "Fib".as_name(12, 15),
                                            vec![ExpressionNode::BinaryExpression(
                                                OperandNode::new(
                                                    Operand::Minus,
                                                    Location::new(12, 21)
                                                ),
                                                Box::new("N".as_var_expr(12, 19)),
                                                Box::new(1.as_lit_expr(12, 23)),
                                            )]
                                        )),
                                        Box::new(ExpressionNode::FunctionCall(
                                            "Fib".as_name(12, 28),
                                            vec![ExpressionNode::BinaryExpression(
                                                OperandNode::new(
                                                    Operand::Minus,
                                                    Location::new(12, 34)
                                                ),
                                                Box::new("N".as_var_expr(12, 32)),
                                                Box::new(2.as_lit_expr(12, 36)),
                                            )]
                                        ))
                                    )
                                )
                            ])
                        })
                    ],
                    pos: Location::new(8, 1)
                }),
            ],
        );
    }
}
