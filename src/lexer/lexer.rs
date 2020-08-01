use super::error::*;
use super::{Keyword, LexemeNode};
use crate::common::*;
use crate::reader::*;
use std::convert::From;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::str::FromStr;

#[derive(Debug)]
pub struct Lexer<T: BufRead> {
    reader: CharOrEofReader<T>,
    pos: Location,
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

impl<T: BufRead> Lexer<T> {
    pub fn new(reader: CharOrEofReader<T>) -> Lexer<T> {
        Lexer {
            reader: reader,
            pos: Location::start(),
        }
    }

    pub fn read(&mut self) -> Result<LexemeNode, LexerError> {
        let peeked = self.peek_one()?;
        match peeked {
            None => {
                self.read_one()?; // consume it so that next invocation yields unexpected eof error
                Ok(LexemeNode::EOF(self.pos))
            },
            Some(peeked) => self.read_char(peeked),
        }
    }

    fn read_char(&mut self, peeked: char) -> Result<LexemeNode, LexerError> {
        let pos = self.pos;
        if is_letter(peeked) {
            let buf = self.read_while(is_alphanumeric)?;
            match Keyword::from_str(&buf) {
                Ok(k) => Ok(LexemeNode::Keyword(k, buf, pos)),
                Err(_) => Ok(LexemeNode::Word(buf, pos)),
            }
        } else if is_whitespace(peeked) {
            let buf = self.read_while(is_whitespace)?;
            Ok(LexemeNode::Whitespace(buf, pos))
        } else if is_digit(peeked) {
            let buf = self.read_while(is_digit)?;
            Ok(LexemeNode::Digits(buf, pos))
        } else if is_eol(peeked) {
            let buf = self.read_while_eol()?;
            Ok(LexemeNode::EOL(buf, pos))
        } else if is_symbol(peeked) {
            self.pos.inc_col();
            self.read_one()?;
            Ok(LexemeNode::Symbol(peeked, pos))
        } else {
            Err(LexerError::UnsupportedCharacter(peeked, pos))
        }
    }

    fn peek_one(&mut self) -> Result<Option<char>, LexerError> {
        self.reader
            .peek()
            .map_err(|e| LexerError::Internal(e.to_string(), self.pos))
    }

    fn read_one(&mut self) -> Result<Option<char>, LexerError> {
        self.reader
            .read()
            .map_err(|e| LexerError::Internal(e.to_string(), self.pos))
    }

    fn read_while(&mut self, predicate: fn(char) -> bool) -> Result<String, LexerError> {
        let mut result: String = String::new();
        let mut next = self.consume_if(predicate)?;
        while next.is_some() {
            result.push(next.unwrap());
            next = self.consume_if(predicate)?;
        }

        Ok(result)
    }

    fn consume_if(&mut self, predicate: fn(char) -> bool) -> Result<Option<char>, LexerError> {
        match self.peek_one()? {
            Some(ch) => {
                if predicate(ch) {
                    self.read_one()?;
                    self.pos.inc_col();
                    Ok(Some(ch))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None)
        }
    }

    fn read_while_eol(&mut self) -> Result<String, LexerError> {
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

// bytes || &str -> Lexer
impl<T> From<T> for Lexer<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        Lexer::new(CharOrEofReader::from(input))
    }
}

// File -> Lexer
impl From<File> for Lexer<BufReader<File>> {
    fn from(input: File) -> Self {
        Lexer::new(CharOrEofReader::from(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "PRINT \"Hello, world!\"";
        let mut lexer = Lexer::from(input);
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Word("PRINT".to_string(), Location::new(1, 1))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Whitespace(" ".to_string(), Location::new(1, 6))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Symbol('"', Location::new(1, 7))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Word("Hello".to_string(), Location::new(1, 8))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Symbol(',', Location::new(1, 13))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Whitespace(" ".to_string(), Location::new(1, 14))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Word("world".to_string(), Location::new(1, 15))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Symbol('!', Location::new(1, 20))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Symbol('"', Location::new(1, 21))
        );
        assert_eq!(lexer.read().unwrap(), LexemeNode::EOF(Location::new(1, 22)));
    }

    #[test]
    fn test_cr_lf() {
        let mut lexer = Lexer::from("Hi\r\n123");
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Word("Hi".to_string(), Location::new(1, 1))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::EOL("\r\n".to_string(), Location::new(1, 3))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Digits("123".to_string(), Location::new(2, 1))
        );
        assert_eq!(lexer.read().unwrap(), LexemeNode::EOF(Location::new(2, 4)));
    }

    #[test]
    fn test_cr_lf_2() {
        let mut lexer = Lexer::from("Hi\r\n\n\r");
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::Word("Hi".to_string(), Location::new(1, 1))
        );
        assert_eq!(
            lexer.read().unwrap(),
            LexemeNode::EOL("\r\n\n\r".to_string(), Location::new(1, 3))
        );
        assert_eq!(lexer.read().unwrap(), LexemeNode::EOF(Location::new(4, 1)));
    }

    #[test]
    fn test_eof_is_only_once() {
        let mut lexer = Lexer::from("Hi");
        assert_eq!(lexer.read().unwrap(), LexemeNode::Word("Hi".to_string(), Location::new(1, 1)));
        assert_eq!(lexer.read().unwrap(), LexemeNode::EOF(Location::new(1, 3)));
        assert_eq!(lexer.read(), Err(LexerError::Internal("unexpected end of file".to_string(), Location::new(1 ,3))));
    }
}
