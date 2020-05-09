use crate::common::*;
use crate::lexer::{Keyword, LexemeNode};
use crate::parser::buf_lexer::BufLexer;
use crate::parser::error::*;
use crate::parser::types::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

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
            v.push(self.parse_top_level_token(next)?);
            next = self.read_skipping_whitespace_and_eol()?
        }
        Ok(v)
    }

    fn parse_top_level_token(
        &mut self,
        next: LexemeNode,
    ) -> Result<TopLevelTokenNode, ParserError> {
        match next {
            LexemeNode::Keyword(k, _, pos) => match k {
                Keyword::Declare => self.demand_declaration().map(|x| x.at(pos)),
                Keyword::DefDbl
                | Keyword::DefInt
                | Keyword::DefLng
                | Keyword::DefSng
                | Keyword::DefStr => self.demand_def_type(k).map(|x| x.at(pos)),
                Keyword::Function => self.demand_function_implementation().map(|x| x.at(pos)),
                Keyword::Sub => self.demand_sub_implementation().map(|x| x.at(pos)),
                Keyword::If
                | Keyword::Input
                | Keyword::For
                | Keyword::While
                | Keyword::Const
                | Keyword::On
                | Keyword::GoTo => self
                    .demand_statement(next)
                    .map(|s| s.consume())
                    .map(|(s, p)| TopLevelToken::from(s).at(p)),
                _ => unexpected("Unexpected top level token", next),
            },
            _ => self
                .demand_statement(next)
                .map(|s| s.consume())
                .map(|(s, p)| TopLevelToken::from(s).at(p)),
        }
    }

    // whitespace and eol

    pub fn read_skipping_whitespace_and_eol(&mut self) -> Result<LexemeNode, ParserError> {
        let next = self.buf_lexer.read()?;
        self.skip_whitespace_and_eol(next)
    }

    pub fn skip_whitespace_and_eol(
        &mut self,
        lexeme: LexemeNode,
    ) -> Result<LexemeNode, ParserError> {
        match lexeme {
            LexemeNode::Whitespace(_, _) | LexemeNode::EOL(_, _) => {
                self.read_skipping_whitespace_and_eol()
            }
            _ => Ok(lexeme),
        }
    }

    // whitespace

    pub fn read_skipping_whitespace(&mut self) -> Result<LexemeNode, ParserError> {
        let next = self.buf_lexer.read()?;
        self.skip_whitespace(next)
    }

    pub fn skip_whitespace(&mut self, lexeme: LexemeNode) -> Result<LexemeNode, ParserError> {
        if lexeme.is_whitespace() {
            self.read_skipping_whitespace()
        } else {
            Ok(lexeme)
        }
    }

    pub fn read_demand_whitespace<S: AsRef<str>>(&mut self, msg: S) -> Result<(), ParserError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Whitespace(_, _) => Ok(()),
            _ => unexpected(msg, next),
        }
    }

    pub fn read_preserve_whitespace(
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

    pub fn read_demand_eol_skipping_whitespace(&mut self) -> Result<(), ParserError> {
        let next = self.read_skipping_whitespace()?;
        match next {
            LexemeNode::EOL(_, _) => Ok(()),
            _ => unexpected("Expected EOL", next),
        }
    }

    pub fn read_demand_eol_or_eof_skipping_whitespace(&mut self) -> Result<(), ParserError> {
        let next = self.read_skipping_whitespace()?;
        match next {
            LexemeNode::EOL(_, _) | LexemeNode::EOF(_) => Ok(()),
            _ => unexpected("Expected EOL or EOF", next),
        }
    }

    // symbol

    pub fn read_demand_symbol_skipping_whitespace(&mut self, ch: char) -> Result<(), ParserError> {
        let next = self.read_skipping_whitespace()?;
        if next.is_symbol(ch) {
            Ok(())
        } else {
            unexpected(format!("Expected {}", ch), next)
        }
    }

    // keyword

    pub fn read_demand_keyword(&mut self, keyword: Keyword) -> Result<(), ParserError> {
        let next = self.buf_lexer.read()?;
        if next.is_keyword(keyword) {
            Ok(())
        } else {
            unexpected("Expected keyword", next)
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
                    vec!["N!".as_name(1, 24)],
                ),
                // PRINT "Enter the number of fibonacci to calculate"
                TopLevelToken::Statement(Statement::SubCall(
                    BareName::from("PRINT"),
                    vec!["Enter the number of fibonacci to calculate".as_lit_expr(2, 7)],
                )),
                // INPUT N
                TopLevelToken::Statement(Statement::SubCall(
                    BareName::from("INPUT"),
                    vec!["N".as_var_expr(3, 7)]
                )),
                // FOR I = 0 TO N
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: "I".as_name(4, 5),
                    lower_bound: 0.as_lit_expr(4, 9),
                    upper_bound: "N".as_var_expr(4, 14),
                    step: None,
                    statements: vec![
                        // PRINT "Fibonacci of ", I, " is ", Fib(I)
                        Statement::SubCall(
                            BareName::from("PRINT"),
                            vec![
                                "Fibonacci of".as_lit_expr(5, 11),
                                "I".as_var_expr(5, 27),
                                "is".as_lit_expr(5, 30),
                                Expression::FunctionCall(
                                    Name::from("Fib"),
                                    vec!["I".as_var_expr(5, 40)],
                                )
                                .at_rc(5, 36),
                            ],
                        )
                        .at_rc(5, 5),
                    ],
                    next_counter: None,
                })),
                // FUNCTION Fib (N)
                TopLevelToken::FunctionImplementation(
                    Name::from("Fib").at_rc(8, 10),
                    vec!["N".as_name(8, 15)],
                    vec![
                        // IF N <= 1 THEN
                        Statement::IfBlock(IfBlockNode {
                            if_block: ConditionalBlockNode {
                                // N <= 1
                                condition: Expression::BinaryExpression(
                                    Operand::LessOrEqualThan,
                                    Box::new("N".as_var_expr(9, 8)),
                                    Box::new(1.as_lit_expr(9, 13))
                                )
                                .at_rc(9, 10),
                                statements: vec![
                                    // Fib = N
                                    Statement::Assignment(
                                        Name::from("Fib"),
                                        "N".as_var_expr(10, 15)
                                    )
                                    .at_rc(10, 9)
                                ],
                            },
                            else_if_blocks: vec![],
                            else_block: Some(vec![
                                // ELSE Fib = Fib(N - 1) + Fib(N - 2)
                                Statement::Assignment(
                                    Name::from("Fib"),
                                    Expression::BinaryExpression(
                                        Operand::Plus,
                                        Box::new(
                                            Expression::FunctionCall(
                                                Name::from("Fib"),
                                                vec![Expression::BinaryExpression(
                                                    Operand::Minus,
                                                    Box::new("N".as_var_expr(12, 19)),
                                                    Box::new(1.as_lit_expr(12, 23)),
                                                )
                                                .at_rc(12, 21)]
                                            )
                                            .at_rc(12, 15)
                                        ),
                                        Box::new(
                                            Expression::FunctionCall(
                                                Name::from("Fib"),
                                                vec![Expression::BinaryExpression(
                                                    Operand::Minus,
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
