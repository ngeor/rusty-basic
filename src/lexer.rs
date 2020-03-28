use crate::common::Result;
use crate::reader::*;
use std::io::prelude::*;

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
            _ => panic!("Cannot format {:?}", self),
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
}

impl<T: BufRead> Lexer<T> {
    pub fn new(reader: T) -> Lexer<T> {
        Lexer {
            reader: CharOrEofReader::new(reader),
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

    pub fn last_pos(&self) -> RowCol {
        self._last_pos
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

pub struct BufLexer<T> {
    lexer: Lexer<T>,
    _history: Vec<Lexeme>,
    _index: usize,
    _mark_index: usize,
}

impl<T: BufRead> BufLexer<T> {
    pub fn new(reader: T) -> BufLexer<T> {
        BufLexer {
            lexer: Lexer::new(reader),
            _history: vec![],
            _index: 0,
            _mark_index: 0,
        }
    }

    /// Reads the next lexeme.
    /// The lexeme is stored and no further reads will be done unless
    /// consume is called.
    pub fn read(&mut self) -> LexerResult {
        if self.needs_to_read() {
            let new_lexeme = self.lexer.read()?;
            self._history.push(new_lexeme);
        }
        Ok(self._history[self._index].clone())
    }

    fn needs_to_read(&self) -> bool {
        self._index >= self._history.len()
    }

    /// Consumes the previously read lexeme, allowing further reads.
    pub fn consume(&mut self) {
        if self._history.is_empty() {
            panic!("No previously read lexeme!");
        } else {
            self._index += 1;
        }
    }

    pub fn mark(&mut self) {
        self._mark_index = self._index;
    }

    pub fn backtrack(&mut self) {
        self._index = self._mark_index;
    }

    pub fn clear(&mut self) {
        while self._index > 0 {
            self._history.remove(0);
            self._index -= 1;
        }
        self._mark_index = 0;
    }

    /// Tries to read the given word. If the next lexeme is this particular word,
    /// it consumes it and it returns true.
    pub fn try_consume_word(&mut self, word: &str) -> Result<bool> {
        let lexeme = self.read()?;
        match lexeme {
            Lexeme::Word(w) => {
                if w == word {
                    self.consume();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    pub fn try_consume_any_word(&mut self) -> Result<Option<String>> {
        let lexeme = self.read()?;
        match lexeme {
            Lexeme::Word(w) => {
                self.consume();
                Ok(Some(w))
            },
            _ => Ok(None)
        }
    }

    pub fn try_consume_symbol(&mut self, ch: char) -> Result<bool> {
        let lexeme = self.read()?;
        match lexeme {
            Lexeme::Symbol(w) => {
                if w == ch {
                    self.consume();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    pub fn demand_any_word(&mut self) -> Result<String> {
        let lexeme = self.read()?;
        match lexeme {
            Lexeme::Word(w) => {
                self.consume();
                Ok(w)
            }
            _ => self.err("Expected word"),
        }
    }

    pub fn demand_specific_word(&mut self, expected: &str) -> Result<()> {
        let word = self.demand_any_word()?;
        if word != expected {
            self.err(format!("Expected {}, found {}", expected, word))
        } else {
            Ok(())
        }
    }

    pub fn demand_symbol(&mut self, ch: char) -> Result<()> {
        let lexeme = self.read()?;
        match lexeme {
            Lexeme::Symbol(s) => {
                if s == ch {
                    self.consume();
                    return Ok(());
                }
            }
            _ => (),
        }

        self.err(format!("Expected symbol {}", ch))
    }

    pub fn demand_eol(&mut self) -> Result<()> {
        let lexeme = self.read()?;
        match lexeme {
            Lexeme::EOL(_) => {
                self.consume();
                Ok(())
            }
            _ => self.err("Expected EOL"),
        }
    }

    pub fn demand_eol_or_eof(&mut self) -> Result<()> {
        let lexeme = self.read()?;
        match lexeme {
            Lexeme::EOL(_) | Lexeme::EOF => {
                self.consume();
                Ok(())
            }
            _ => self.err("Expected EOL or EOF"),
        }
    }

    pub fn demand_whitespace(&mut self) -> Result<()> {
        let lexeme = self.read()?;
        match lexeme {
            Lexeme::Whitespace(_) => {
                self.consume();
                Ok(())
            }
            _ => self.err("Expected whitespace"),
        }
    }

    /// Reads and consumes while the next lexeme is Whitespace.
    ///
    /// Returns true if at least one Whitespace was found, false otherwise.
    pub fn skip_whitespace(&mut self) -> Result<bool> {
        let mut found = false;
        loop {
            let lexeme = self.read()?;
            match lexeme {
                Lexeme::Whitespace(_) => {
                    found = true;
                    self.consume();
                }
                _ => break,
            }
        }
        Ok(found)
    }

    pub fn skip_whitespace_and_eol(&mut self) -> Result<()> {
        loop {
            let lexeme = self.read()?;
            match lexeme {
                Lexeme::Whitespace(_) | Lexeme::EOL(_) => self.consume(),
                _ => break,
            }
        }
        Ok(())
    }

    pub fn err<TResult, S: AsRef<str>>(&self, msg: S) -> Result<TResult> {
        if self.needs_to_read() {
            self.lexer.err(msg)
        } else {
            self.lexer.err(format!(
                "{} {:?}",
                msg.as_ref(),
                self._history[self._index].clone()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    impl<T> From<T> for Lexer<BufReader<Cursor<T>>>
    where
        T: std::convert::AsRef<[u8]>,
    {
        fn from(input: T) -> Lexer<BufReader<Cursor<T>>> {
            let c = Cursor::new(input);
            let reader = BufReader::new(c);
            Lexer::new(reader)
        }
    }

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
