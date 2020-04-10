use super::error::*;
use super::{Lexeme, LexerResult};
use crate::common::Location;
use crate::reader::*;
use std::convert::From;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

#[derive(Debug)]
pub struct Lexer<T> {
    reader: CharOrEofReader<T>,
    pos: Location,
}

fn _is_letter(ch: char) -> bool {
    (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z')
}

fn _is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
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
}

impl<T: BufRead> Lexer<T> {
    pub fn new(reader: CharOrEofReader<T>) -> Lexer<T> {
        Lexer {
            reader: reader,
            pos: Location::start(),
        }
    }

    pub fn pos(&self) -> Location {
        self.pos
    }

    pub fn read(&mut self) -> LexerResult {
        let x = self._read_one()?;
        match x {
            None => Ok(Lexeme::EOF),
            Some(ch) => self._read_char(ch),
        }
    }

    fn _read_char(&mut self, ch: char) -> LexerResult {
        if _is_letter(ch) {
            let buf = self._read_while(_is_letter)?;
            Ok(Lexeme::Word(buf))
        } else if _is_whitespace(ch) {
            let buf = self._read_while(_is_whitespace)?;
            Ok(Lexeme::Whitespace(buf))
        } else if _is_digit(ch) {
            let buf = self._read_while(_is_digit)?;
            let num: u32 = self._parse_digits(buf)?;
            Ok(Lexeme::Digits(num))
        } else if _is_eol(ch) {
            let buf = self._read_while_eol()?;
            Ok(Lexeme::EOL(buf))
        } else if _is_symbol(ch) {
            self._consume();
            self.pos.inc_col();
            Ok(Lexeme::Symbol(ch))
        } else {
            Err(LexerError::UnsupportedCharacter(ch, self.pos))
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
        assert_eq!(lexer.read().unwrap(), Lexeme::Word("PRINT".to_string()));
        assert_eq!(lexer.read().unwrap(), Lexeme::Whitespace(" ".to_string()));
        assert_eq!(lexer.read().unwrap(), Lexeme::Symbol('"'));
        assert_eq!(lexer.read().unwrap(), Lexeme::Word("Hello".to_string()));
        assert_eq!(lexer.read().unwrap(), Lexeme::Symbol(','));
        assert_eq!(lexer.read().unwrap(), Lexeme::Whitespace(" ".to_string()));
        assert_eq!(lexer.read().unwrap(), Lexeme::Word("world".to_string()));
        assert_eq!(lexer.read().unwrap(), Lexeme::Symbol('!'));
        assert_eq!(lexer.read().unwrap(), Lexeme::Symbol('"'));
        assert_eq!(lexer.read().unwrap(), Lexeme::EOF);
    }

    #[test]
    fn test_cr_lf() {
        let input = "Hi\r\n\n\r";
        let mut lexer = Lexer::from(input);
        assert_eq!(lexer.read().unwrap(), Lexeme::Word("Hi".to_string()));
        assert_eq!(lexer.read().unwrap(), Lexeme::EOL("\r\n\n\r".to_string()));
        assert_eq!(lexer.read().unwrap(), Lexeme::EOF);
    }
}
