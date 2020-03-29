use crate::common::Result;
use crate::reader::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

pub type LexerResult = Result<Lexeme>;

#[derive(Debug, PartialEq, Clone)]
pub enum Lexeme {
    /// EOF
    EOF,

    /// CR, LF
    EOL(String),

    /// A sequence of letters (A-Z or a-z)
    Word(String),

    /// A sequence of whitespace (spaces and tabs)
    Whitespace(String),

    /// A punctuation symbol
    Symbol(char),

    /// An integer number
    Digits(u32),
}

impl Lexeme {
    pub fn push_to(&self, buf: &mut String) {
        match self {
            Self::Word(s) => buf.push_str(s),
            Self::Whitespace(s) => buf.push_str(s),
            Self::Symbol(c) => buf.push(*c),
            _ => unimplemented!(),
        }
    }
}

pub struct Lexer<T> {
    reader: CharOrEofReader<T>,
    _last_pos: RowCol,
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
            _last_pos: RowCol::new(),
        }
    }

    pub fn read(&mut self) -> LexerResult {
        self._last_pos = self.reader.pos();
        let x = self.reader.read_and_consume()?;
        match x {
            CharOrEof::EOF => Ok(Lexeme::EOF),
            CharOrEof::Char(ch) => {
                if _is_letter(ch) {
                    Ok(Lexeme::Word(self._read_while(ch, _is_letter)?))
                } else if _is_whitespace(ch) {
                    Ok(Lexeme::Whitespace(self._read_while(ch, _is_whitespace)?))
                } else if _is_digit(ch) {
                    Ok(Lexeme::Digits(
                        (self._read_while(ch, _is_digit)?).parse::<u32>().unwrap(),
                    ))
                } else if _is_eol(ch) {
                    Ok(Lexeme::EOL(self._read_while(ch, _is_eol)?))
                } else if _is_symbol(ch) {
                    Ok(Lexeme::Symbol(ch))
                } else {
                    self.err(format!("Unexpected character {}", ch))
                }
            }
        }
    }

    pub fn err<TResult, S: AsRef<str>>(&self, msg: S) -> Result<TResult> {
        Err(format!(
            "[lexer] Line {} Column {}: {}",
            self._last_pos.row(),
            self._last_pos.col(),
            msg.as_ref()
        ))
    }

    fn _read_while(&mut self, initial: char, predicate: fn(char) -> bool) -> Result<String> {
        let mut result: String = String::new();
        result.push(initial);

        loop {
            let x = self.reader.read()?;
            match x {
                CharOrEof::Char(ch) => {
                    if predicate(ch) {
                        result.push(ch);
                        self.reader.consume()?;
                    } else {
                        break;
                    }
                }
                CharOrEof::EOF => {
                    break;
                }
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
