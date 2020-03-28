use crate::common::Result;
use crate::lexer::*;
use std::io::prelude::*;

mod assignment;
mod declaration;
mod expression;
mod for_loop;
mod function_implementation;
mod if_block;
mod qname;
mod statement;
mod sub_call;

pub use self::expression::*;
pub use self::if_block::*;
pub use self::qname::*;
pub use self::statement::*;

pub type Block = Vec<Statement>;

#[derive(Debug, PartialEq)]
pub enum TopLevelToken {
    EOF,
    FunctionDeclaration(NameWithTypeQualifier, Vec<NameWithTypeQualifier>),
    Statement(Statement),
    FunctionImplementation(NameWithTypeQualifier, Vec<NameWithTypeQualifier>, Block),
}

impl TopLevelToken {
    pub fn sub_call<S: AsRef<str>>(name: S, args: Vec<Expression>) -> TopLevelToken {
        TopLevelToken::Statement(Statement::sub_call(name, args))
    }
}

pub type Program = Vec<TopLevelToken>;

pub struct Parser<T> {
    buf_lexer: BufLexer<T>,
}

impl<T: BufRead> Parser<T> {
    pub fn new(reader: T) -> Parser<T> {
        Parser {
            buf_lexer: BufLexer::new(reader),
        }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let mut v: Vec<TopLevelToken> = vec![];
        loop {
            let x = self._parse_top_level_token()?;
            match x {
                TopLevelToken::EOF => break,
                _ => v.push(x),
            };
        }
        Ok(v)
    }

    fn _parse_top_level_token(&mut self) -> Result<TopLevelToken> {
        if let Some(d) = self.try_parse_declaration()? {
            Ok(d)
        } else if let Some(f) = self.try_parse_function_implementation()? {
            Ok(f)
        } else if let Some(s) = self._try_parse_statement_as_top_level_token()? {
            Ok(s)
        } else {
            let lexeme = self.buf_lexer.read()?;
            match lexeme {
                Lexeme::EOF => {
                    self.buf_lexer.consume();
                    Ok(TopLevelToken::EOF)
                }
                _ => self.buf_lexer.err("[parser] Unexpected lexeme"),
            }
        }
    }

    fn _try_parse_statement_as_top_level_token(&mut self) -> Result<Option<TopLevelToken>> {
        match self.try_parse_statement()? {
            Some(statement) => Ok(Some(TopLevelToken::Statement(statement))),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod test_utils {
    use super::*;
    use std::fs::File;
    use std::io::{BufReader, Cursor};

    // bytes || &str -> Parser
    impl<T> From<T> for Parser<std::io::BufReader<std::io::Cursor<T>>>
    where
        T: std::convert::AsRef<[u8]>,
    {
        fn from(input: T) -> Self {
            Parser::new(BufReader::new(Cursor::new(input)))
        }
    }

    // File -> Parser
    impl From<File> for Parser<std::io::BufReader<File>> {
        fn from(input: File) -> Self {
            Parser::new(BufReader::new(input))
        }
    }

    pub fn parse<T>(input: T) -> Result<Program>
    where
        T: std::convert::AsRef<[u8]>,
    {
        let mut parser = Parser::from(input);
        parser.parse()
    }

    pub fn parse_file(filename: &str) -> Program {
        let file_path = format!("fixtures/{}", filename);
        let mut parser = Parser::from(File::open(file_path).expect("Could not read bas file"));
        parser.parse().expect("Could not parse program")
    }
}

#[cfg(test)]
mod tests {
    use super::test_utils::*;
    use super::*;
    use crate::parser::if_block::*;

    #[test]
    fn test_parse_sub_call_no_args() {
        let input = "PRINT";
        let program = parse(input).unwrap();
        assert_eq!(program, vec![TopLevelToken::sub_call("PRINT", vec![])]);
    }

    #[test]
    fn test_parse_sub_call_single_arg_string_literal() {
        let input = "PRINT \"Hello, world!\"";
        let program = parse(input).unwrap();
        assert_eq!(
            program,
            vec![TopLevelToken::sub_call(
                "PRINT",
                vec![Expression::string_literal("Hello, world!")]
            )]
        );
    }

    #[test]
    fn test_parse_fixture_hello1() {
        let program = parse_file("HELLO1.BAS");
        assert_eq!(
            program,
            vec![TopLevelToken::sub_call(
                "PRINT",
                vec![Expression::string_literal("Hello, world!")]
            )]
        );
    }

    #[test]
    fn test_parse_fixture_hello2() {
        let program = parse_file("HELLO2.BAS");
        assert_eq!(
            program,
            vec![TopLevelToken::sub_call(
                "PRINT",
                vec![
                    Expression::string_literal("Hello"),
                    Expression::string_literal("world!"),
                ]
            )]
        );
    }

    #[test]
    fn test_parse_fixture_hello_system() {
        let program = parse_file("HELLO_S.BAS");
        assert_eq!(
            program,
            vec![
                TopLevelToken::sub_call(
                    "PRINT",
                    vec![Expression::string_literal("Hello, world!"),]
                ),
                TopLevelToken::sub_call("SYSTEM", vec![])
            ]
        );
    }

    #[test]
    fn test_parse_fixture_input() {
        let program = parse_file("INPUT.BAS");
        assert_eq!(
            program,
            vec![
                TopLevelToken::sub_call("INPUT", vec![Expression::variable_name_unqualified("N")]),
                TopLevelToken::sub_call("PRINT", vec![Expression::variable_name_unqualified("N")])
            ]
        );
    }

    #[test]
    fn test_parse_fixture_fib() {
        let program = parse_file("FIB.BAS");
        assert_eq!(
            program,
            vec![
                // DECLARE FUNCTION Fib! (N!)
                TopLevelToken::FunctionDeclaration(
                    NameWithTypeQualifier::new("Fib", TypeQualifier::BangInteger),
                    vec![NameWithTypeQualifier::new("N", TypeQualifier::BangInteger)]
                ),
                // PRINT "Enter the number of fibonacci to calculate"
                TopLevelToken::sub_call(
                    "PRINT",
                    vec![Expression::string_literal(
                        "Enter the number of fibonacci to calculate"
                    )]
                ),
                // INPUT N
                TopLevelToken::sub_call("INPUT", vec![Expression::variable_name_unqualified("N")]),
                // FOR I = 0 TO N
                TopLevelToken::Statement(Statement::ForLoop(
                    NameWithTypeQualifier::new_unqualified("I"),
                    Expression::IntegerLiteral(0),
                    Expression::variable_name_unqualified("N"),
                    vec![
                        // PRINT "Fibonacci of ", I, " is ", Fib(I)
                        Statement::sub_call(
                            "PRINT",
                            vec![
                                Expression::string_literal("Fibonacci of "),
                                Expression::variable_name_unqualified("I"),
                                Expression::string_literal(" is "),
                                Expression::FunctionCall(
                                    "Fib".to_string(),
                                    vec![Expression::variable_name_unqualified("I")]
                                )
                            ]
                        )
                    ]
                )),
                // FUNCTION Fib (N)
                TopLevelToken::FunctionImplementation(
                    NameWithTypeQualifier::new_unqualified("Fib"),
                    vec![NameWithTypeQualifier::new_unqualified("N")],
                    vec![
                        // IF N <= 1 THEN
                        Statement::IfBlock(IfBlock::new_if_else(
                            // N <= 1
                            Expression::lte(
                                Expression::variable_name_unqualified("N"),
                                Expression::IntegerLiteral(1)
                            ),
                            // Fib = N
                            vec![Statement::Assignment(
                                NameWithTypeQualifier::new_unqualified("Fib"),
                                Expression::variable_name_unqualified("N")
                            )],
                            // ELSE Fib = Fib(N - 1) + Fib(N - 2)
                            vec![Statement::Assignment(
                                NameWithTypeQualifier::new_unqualified("Fib"),
                                Expression::plus(
                                    Expression::FunctionCall(
                                        "Fib".to_string(),
                                        vec![Expression::minus(
                                            Expression::variable_name_unqualified("N"),
                                            Expression::IntegerLiteral(1)
                                        )]
                                    ),
                                    Expression::FunctionCall(
                                        "Fib".to_string(),
                                        vec![Expression::minus(
                                            Expression::variable_name_unqualified("N"),
                                            Expression::IntegerLiteral(2)
                                        )]
                                    )
                                )
                            )]
                        ))
                    ]
                )
            ]
        );
    }
}
