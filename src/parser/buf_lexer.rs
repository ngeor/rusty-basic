use super::ParserError;
use crate::lexer::{LexemeNode, Lexer};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

#[derive(Debug)]
pub struct BufLexer<T: BufRead> {
    lexer: Lexer<T>,
    history: VecDeque<LexemeNode>,
}

impl<T: BufRead> BufLexer<T> {
    pub fn new(lexer: Lexer<T>) -> BufLexer<T> {
        BufLexer {
            lexer: lexer,
            history: VecDeque::new(),
        }
    }

    fn _lexer_read(&mut self) -> Result<LexemeNode, ParserError> {
        self.lexer.read().map_err(|e| ParserError::LexerError(e))
    }

    pub fn read(&mut self) -> Result<LexemeNode, ParserError> {
        match self.history.pop_front() {
            Some(x) => Ok(x),
            None => self._lexer_read(),
        }
    }

    pub fn skip_if<F>(&mut self, f: F) -> Result<bool, ParserError>
    where
        F: Fn(&LexemeNode) -> bool,
    {
        let next = self.read()?;
        if f(&next) {
            Ok(true)
        } else {
            self.undo(next);
            Ok(false)
        }
    }

    pub fn try_read<F, TR, E>(&mut self, f: F) -> Result<Option<TR>, ParserError>
    where
        F: Fn(&LexemeNode) -> std::result::Result<TR, E>,
    {
        let next = self.read()?;
        match f(&next) {
            Ok(x) => Ok(Some(x)),
            Err(_) => {
                self.undo(next);
                Ok(None)
            }
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

pub trait BufLexerUndo<T> {
    fn undo(&mut self, lexeme_like: T);
}

impl<T: BufRead> BufLexerUndo<LexemeNode> for BufLexer<T> {
    fn undo(&mut self, lexeme: LexemeNode) {
        self.history.push_front(lexeme);
    }
}

impl<T: BufRead> BufLexerUndo<Option<LexemeNode>> for BufLexer<T> {
    fn undo(&mut self, opt_lexeme: Option<LexemeNode>) {
        match opt_lexeme {
            Some(lexeme) => self.undo(lexeme),
            None => (),
        }
    }
}
