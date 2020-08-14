use super::{Keyword, Lexeme, LexemeNode};
use crate::char_reader::*;
use crate::common::*;
use std::convert::From;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::str::FromStr;

/// A Lexer uses a CharReader to turn characters into LexemeNodes.
#[derive(Debug)]
pub struct Lexer<T: BufRead> {
    reader: CharReader<T>,
    pos: Location,
    seen_eof: bool,
}

impl<T: BufRead> Lexer<T> {
    pub fn new(reader: CharReader<T>) -> Lexer<T> {
        Lexer {
            reader: reader,
            pos: Location::start(),
            seen_eof: false,
        }
    }

    fn read_char(&mut self, peeked: char) -> Result<LexemeNode, QErrorNode> {
        let pos = self.pos;
        if is_letter(peeked) {
            let buf = self.read_while(is_alphanumeric)?;
            match Keyword::from_str(&buf) {
                Ok(k) => Ok(Lexeme::Keyword(k, buf).at(pos)),
                Err(_) => Ok(Lexeme::Word(buf).at(pos)),
            }
        } else if is_whitespace(peeked) {
            let buf = self.read_while(is_whitespace)?;
            Ok(Lexeme::Whitespace(buf).at(pos))
        } else if is_digit(peeked) {
            let buf = self.read_while(is_digit)?;
            Ok(Lexeme::Digits(buf).at(pos))
        } else if is_eol(peeked) {
            let buf = self.read_while_eol()?;
            Ok(Lexeme::EOL(buf).at(pos))
        } else if is_symbol(peeked) {
            self.pos.inc_col();
            self.read_one()?;
            Ok(Lexeme::Symbol(peeked).at(pos))
        } else {
            Err(QError::UnsupportedCharacter(peeked)).with_err_at(pos)
        }
    }

    fn peek_one(&mut self) -> Result<Option<char>, QErrorNode> {
        self.reader
            .peek_ng()
            .map_err(|e| e.into())
            .with_err_at(self.pos)
    }

    fn read_one(&mut self) -> Result<Option<char>, QErrorNode> {
        self.reader
            .read_ng()
            .map_err(|e| e.into())
            .with_err_at(self.pos)
    }

    fn read_while(&mut self, predicate: fn(char) -> bool) -> Result<String, QErrorNode> {
        let mut result: String = String::new();
        let mut next = self.consume_if(predicate)?;
        while next.is_some() {
            result.push(next.unwrap());
            next = self.consume_if(predicate)?;
        }

        Ok(result)
    }

    fn consume_if(&mut self, predicate: fn(char) -> bool) -> Result<Option<char>, QErrorNode> {
        match self.peek_one()? {
            Some(ch) => {
                let c = ch;
                if predicate(c) {
                    self.read_one()?;
                    self.pos.inc_col();
                    Ok(Some(c))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    fn read_while_eol(&mut self) -> Result<String, QErrorNode> {
        let mut result: String = String::new();
        let mut previous_was_cr = false;
        loop {
            let x = self.peek_one()?;
            match x {
                Some(ch) => {
                    if ch == '\n' {
                        // \n
                        result.push(ch);
                        self.read_one()?;
                        if previous_was_cr {
                            previous_was_cr = false;
                        } else {
                            self.pos.inc_row();
                        }
                    } else if ch == '\r' {
                        // \r\?
                        result.push(ch);
                        self.read_one()?;
                        self.pos.inc_row();
                        previous_was_cr = true;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(result)
    }
}

impl<T: BufRead> ReadOpt for Lexer<T> {
    type Item = LexemeNode;
    type Err = QErrorNode;

    fn read_ng(&mut self) -> Result<Option<LexemeNode>, QErrorNode> {
        let pos = self.pos();
        let peeked = self.peek_one()?;
        match peeked {
            None => {
                if self.seen_eof {
                    Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into())
                        .with_err_at(pos)
                } else {
                    self.seen_eof = true;
                    Ok(Some(Lexeme::EOF.at(pos)))
                }
            }
            Some(peeked) => {
                let ch = peeked;
                let lexeme_node = self.read_char(ch)?;
                Ok(Some(lexeme_node))
            }
        }
    }
}

/// Rejects Ok(None) values
pub trait DemandTrait<T, E> {
    fn demand<S: AsRef<str>>(self, err_msg: S, err_pos: Location) -> Result<T, E>;
}

impl<T> DemandTrait<T, QErrorNode> for Result<Option<T>, QErrorNode> {
    fn demand<S: AsRef<str>>(self, err_msg: S, err_pos: Location) -> Result<T, QErrorNode> {
        match self {
            Ok(None) => {
                Err(QError::SyntaxError(format!("{}", err_msg.as_ref()))).with_err_at(err_pos)
            }
            Ok(Some(x)) => Ok(x),
            Err(err) => Err(err),
        }
    }
}

// bytes || &str -> Lexer
impl<T> From<T> for Lexer<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        Lexer::new(CharReader::from(input))
    }
}

// File -> Lexer
impl From<File> for Lexer<BufReader<File>> {
    fn from(input: File) -> Self {
        Lexer::new(CharReader::from(input))
    }
}

impl<T: BufRead> HasLocation for Lexer<T> {
    /// Gets the location of the lexeme that will be read next.
    fn pos(&self) -> Location {
        self.pos
    }
}

fn is_letter(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')
}

fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

fn is_alphanumeric(ch: char) -> bool {
    is_letter(ch) || is_digit(ch)
}

fn is_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\t'
}

fn is_eol(ch: char) -> bool {
    ch == '\r' || ch == '\n'
}

fn is_symbol(ch: char) -> bool {
    (ch > ' ' && ch < '0')
        || (ch > '9' && ch < 'A')
        || (ch > 'Z' && ch < 'a')
        || (ch > 'z' && ch <= '~')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "PRINT \"Hello, world!\"";
        let mut lexer = Lexer::from(input);
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Word("PRINT".to_string()).at_rc(1, 1)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Whitespace(" ".to_string()).at_rc(1, 6)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Symbol('"').at_rc(1, 7)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Word("Hello".to_string()).at_rc(1, 8)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Symbol(',').at_rc(1, 13)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Whitespace(" ".to_string()).at_rc(1, 14)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Word("world".to_string()).at_rc(1, 15)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Symbol('!').at_rc(1, 20)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Symbol('"').at_rc(1, 21)
        );
        assert_eq!(lexer.read_ng().unwrap().unwrap(), Lexeme::EOF.at_rc(1, 22));
    }

    #[test]
    fn test_cr_lf() {
        let mut lexer = Lexer::from("Hi\r\n123");
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Word("Hi".to_string()).at_rc(1, 1)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::EOL("\r\n".to_string()).at_rc(1, 3)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Digits("123".to_string()).at_rc(2, 1)
        );
        assert_eq!(lexer.read_ng().unwrap().unwrap(), Lexeme::EOF.at_rc(2, 4));
    }

    #[test]
    fn test_cr_lf_2() {
        let mut lexer = Lexer::from("Hi\r\n\n\r");
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Word("Hi".to_string()).at_rc(1, 1)
        );
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::EOL("\r\n\n\r".to_string()).at_rc(1, 3)
        );
        assert_eq!(lexer.read_ng().unwrap().unwrap(), Lexeme::EOF.at_rc(4, 1));
    }

    #[test]
    fn test_eof_is_only_once() {
        let mut lexer = Lexer::from("Hi");
        assert_eq!(
            lexer.read_ng().unwrap().unwrap(),
            Lexeme::Word("Hi".to_string()).at_rc(1, 1)
        );
        assert_eq!(lexer.read_ng().unwrap().unwrap(), Lexeme::EOF.at_rc(1, 3));
        assert_eq!(
            lexer.read_ng(),
            Err(QError::InputPastEndOfFile).with_err_at_rc(1, 3)
        );
    }

    #[test]
    fn test_location() {
        let mut lexer = Lexer::from("PRINT 1");
        assert_eq!(lexer.pos(), Location::new(1, 1));
        lexer
            .read_ng()
            .unwrap()
            .expect("Read should succeed (PRINT)");
        assert_eq!(lexer.pos(), Location::new(1, 6));
        lexer
            .read_ng()
            .unwrap()
            .expect("Read should succeed (whitespace)");
        assert_eq!(lexer.pos(), Location::new(1, 7));
        lexer.read_ng().unwrap().expect("Read should succeed (1)");
        assert_eq!(lexer.pos(), Location::new(1, 8));
        lexer.read_ng().unwrap().expect("Read should succeed (EOF)");
        assert_eq!(lexer.pos(), Location::new(1, 8));
        lexer.read_ng().expect_err("Read should fail");
    }
}
