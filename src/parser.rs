use crate::lexer::{Keyword, LexemeNode};
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

mod assignment;
mod buf_lexer;
mod declaration;
mod def_type;
mod error;
mod expression;
mod for_loop;
mod function_implementation;
mod if_block;
mod name;
mod statement;
mod sub_call;
mod types;
mod while_wend;

#[cfg(test)]
mod test_utils;

pub use self::buf_lexer::*;
pub use self::error::*;
pub use self::expression::*;
pub use self::for_loop::*;
pub use self::if_block::*;
pub use self::name::*;
pub use self::statement::*;
pub use self::types::*;

#[derive(Debug)]
pub struct Parser<T: BufRead> {
    pub buf_lexer: BufLexer<T>,
}

impl<T: BufRead> Parser<T> {
    pub fn new(buf_lexer: BufLexer<T>) -> Parser<T> {
        Parser { buf_lexer }
    }

    pub fn parse(&mut self) -> Result<ProgramNode, ParserError> {
        let mut v: Vec<TopLevelTokenNode> = vec![];
        let mut next = self.read_skipping_whitespace_and_eol()?;
        while !next.is_eof() {
            v.push(self._parse_top_level_token(next)?);
            next = self.read_skipping_whitespace_and_eol()?
        }
        Ok(v)
    }

    fn _parse_top_level_token(
        &mut self,
        next: LexemeNode,
    ) -> Result<TopLevelTokenNode, ParserError> {
        match next {
            LexemeNode::Keyword(k, _, pos) => match k {
                Keyword::Declare => self.demand_declaration(pos),
                Keyword::DefDbl
                | Keyword::DefInt
                | Keyword::DefLng
                | Keyword::DefSng
                | Keyword::DefStr => self.demand_def_type(k, pos),
                Keyword::Function => self.demand_function_implementation(pos),
                Keyword::If | Keyword::For | Keyword::While => self
                    .demand_statement(next)
                    .map(|s| TopLevelTokenNode::Statement(s)),
                _ => unexpected("Unexpected top level token", next),
            },
            _ => self
                .demand_statement(next)
                .map(|s| TopLevelTokenNode::Statement(s)),
        }
    }

    // whitespace and eol

    fn read_skipping_whitespace_and_eol(&mut self) -> Result<LexemeNode, ParserError> {
        let next = self.buf_lexer.read()?;
        self.skip_whitespace_and_eol(next)
    }

    fn skip_whitespace_and_eol(&mut self, lexeme: LexemeNode) -> Result<LexemeNode, ParserError> {
        match lexeme {
            LexemeNode::Whitespace(_, _) | LexemeNode::EOL(_, _) => {
                self.read_skipping_whitespace_and_eol()
            }
            _ => Ok(lexeme),
        }
    }

    // whitespace

    fn read_skipping_whitespace(&mut self) -> Result<LexemeNode, ParserError> {
        let next = self.buf_lexer.read()?;
        self.skip_whitespace(next)
    }

    fn skip_whitespace(&mut self, lexeme: LexemeNode) -> Result<LexemeNode, ParserError> {
        if lexeme.is_whitespace() {
            self.read_skipping_whitespace()
        } else {
            Ok(lexeme)
        }
    }

    fn read_demand_whitespace<S: AsRef<str>>(&mut self, msg: S) -> Result<(), ParserError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Whitespace(_, _) => Ok(()),
            _ => unexpected(msg, next),
        }
    }

    fn read_preserve_whitespace(
        &mut self,
    ) -> Result<(Option<LexemeNode>, LexemeNode), ParserError> {
        let first = self.buf_lexer.read()?;
        if first.is_whitespace() {
            Ok((Some(first), self.buf_lexer.read()?))
        } else {
            Ok((None, first))
        }
    }

    // eol

    fn read_demand_eol_skipping_whitespace(&mut self) -> Result<(), ParserError> {
        let next = self.read_skipping_whitespace()?;
        match next {
            LexemeNode::EOL(_, _) => Ok(()),
            _ => unexpected("Expected EOL", next),
        }
    }

    fn read_demand_eol_or_eof_skipping_whitespace(&mut self) -> Result<(), ParserError> {
        let next = self.read_skipping_whitespace()?;
        match next {
            LexemeNode::EOL(_, _) | LexemeNode::EOF(_) => Ok(()),
            _ => unexpected("Expected EOL or EOF", next),
        }
    }

    // symbol

    fn read_demand_symbol_skipping_whitespace(&mut self, ch: char) -> Result<(), ParserError> {
        let next = self.read_skipping_whitespace()?;
        if next.is_symbol(ch) {
            Ok(())
        } else {
            unexpected(format!("Expected {}", ch), next)
        }
    }

    // keyword

    fn read_demand_keyword(&mut self, keyword: Keyword) -> Result<(), ParserError> {
        let next = self.buf_lexer.read()?;
        if next.is_keyword(keyword) {
            Ok(())
        } else {
            unexpected("Expected keyword", next)
        }
    }
}

fn unexpected<T, S: AsRef<str>>(msg: S, lexeme: LexemeNode) -> Result<T, ParserError> {
    Err(ParserError::Unexpected(msg.as_ref().to_string(), lexeme))
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
                                pos: Location::new(9, 5),
                                // N <= 1
                                condition: ExpressionNode::BinaryExpression(
                                    OperandNode::new(
                                        Operand::LessOrEqualThan,
                                        Location::new(9, 10)
                                    ),
                                    Box::new("N".as_var_expr(9, 8)),
                                    Box::new(1.as_lit_expr(9, 13))
                                ),
                                statements: vec![
                                    // Fib = N
                                    StatementNode::Assignment(
                                        "Fib".as_name(10, 9),
                                        "N".as_var_expr(10, 15)
                                    )
                                ],
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
