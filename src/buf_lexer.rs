use crate::common::Result;
use crate::lexer::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

#[derive(Debug)]
pub struct BufLexer<T> {
    lexer: Lexer<T>,
    _history: Vec<Lexeme>,
    _index: usize,
    _mark_index: usize,
}

impl<T: BufRead> BufLexer<T> {
    pub fn new(lexer: Lexer<T>) -> BufLexer<T> {
        BufLexer {
            lexer: lexer,
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
            }
            _ => Ok(None),
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

// bytes || &str -> BufLexer
impl<T> From<T> for BufLexer<BufReader<Cursor<T>>>
where
    T: AsRef<[u8]>,
{
    fn from(input: T) -> Self {
        BufLexer::new(Lexer::from(input))
    }
}

// File -> BufLexer
impl From<File> for BufLexer<BufReader<File>> {
    fn from(input: File) -> Self {
        BufLexer::new(Lexer::from(input))
    }
}
