use super::error::*;
use super::{Keyword, LexemeNode};
use crate::common::Location;
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

fn _is_letter(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')
}

fn _is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

fn _is_alphanumeric(ch: char) -> bool {
    _is_letter(ch) || _is_digit(ch)
}

fn _is_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\t'
}

fn _is_eol(ch: char) -> bool {
    ch == '\r' || ch == '\n'
}

fn _is_symbol(ch: char) -> bool {
    ch == '"'
        || ch == '\''
        || ch == '!'
        || ch == ','
        || ch == '$'
        || ch == '%'
        || ch == '+'
        || ch == '-'
        || ch == '*'
        || ch == '/'
        || ch == '('
        || ch == ')'
        || ch == '='
        || ch == '<'
        || ch == '>'
        || ch == '.'
        || ch == '#'
        || ch == '&'
        || ch == '\''
        || ch == ':'
}

impl<T: BufRead> Lexer<T> {
    pub fn new(reader: CharOrEofReader<T>) -> Lexer<T> {
        Lexer {
            reader: reader,
            pos: Location::start(),
        }
    }

    pub fn read(&mut self) -> Result<LexemeNode, LexerError> {
        let x = self._read_one()?;
        match x {
            None => Ok(LexemeNode::EOF(self.pos)),
            Some(ch) => self._read_char(ch),
        }
    }

    fn _read_char(&mut self, ch: char) -> Result<LexemeNode, LexerError> {
        let pos = self.pos;
        if _is_letter(ch) {
            let buf = self._read_while(_is_alphanumeric)?;
            match Keyword::from_str(&buf) {
                Ok(k) => Ok(LexemeNode::Keyword(k, buf, pos)),
                Err(_) => Ok(LexemeNode::Word(buf, pos)),
            }
        } else if _is_whitespace(ch) {
            let buf = self._read_while(_is_whitespace)?;
            Ok(LexemeNode::Whitespace(buf, pos))
        } else if _is_digit(ch) {
            let buf = self._read_while(_is_digit)?;
            let num: u32 = self._parse_digits(buf)?;
            Ok(LexemeNode::Digits(num, pos))
        } else if _is_eol(ch) {
            let buf = self._read_while_eol()?;
            Ok(LexemeNode::EOL(buf, pos))
        } else if _is_symbol(ch) {
            self._consume();
            self.pos.inc_col();
            Ok(LexemeNode::Symbol(ch, pos))
        } else {
            Err(LexerError::UnsupportedCharacter(ch, pos))
        }
    }

    fn _parse_digits(&self, buf: String) -> Result<u32, LexerError> {
        buf.parse::<u32>()
            .map_err(|e| LexerError::Internal(e.to_string(), self.pos))
    }

    fn _read_one(&mut self) -> Result<Option<char>, LexerError> {
        self.reader
            .read()
            .map_err(|e| LexerError::Internal(e.to_string(), self.pos))
    }

    fn _consume(&mut self) -> Option<char> {
        self.reader.consume()
    }

    fn _read_while(&mut self, predicate: fn(char) -> bool) -> Result<String, LexerError> {
        let mut result: String = String::new();

        let mut next = self._consume_if(predicate)?;
        while next.is_some() {
            result.push(next.unwrap());
            next = self._consume_if(predicate)?;
        }

        Ok(result)
    }

    fn _consume_if(&mut self, predicate: fn(char) -> bool) -> Result<Option<char>, LexerError> {
        self.reader
            .read()
            .map(|opt| {
                opt.and_then(|ch| {
                    if predicate(ch) {
                        self._consume();
                        self.pos.inc_col();
                        Some(ch)
                    } else {
                        None
                    }
                })
            })
            .map_err(|e| LexerError::Internal(e.to_string(), self.pos))
    }

    fn _read_while_eol(&mut self) -> Result<String, LexerError> {
        let mut result: String = String::new();
        let mut previous_was_cr = false;
        loop {
            let x = self._read_one()?;
            match x {
                Some(ch) => {
                    if ch == '\n' {
                        // \n
                        result.push(ch);
                        self._consume();
                        if previous_was_cr {
                            previous_was_cr = false;
                        } else {
                            self.pos.inc_row();
                        }
                    } else if ch == '\r' {
                        // \r\?
                        result.push(ch);
                        self._consume();
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
            LexemeNode::Digits(123, Location::new(2, 1))
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
}
