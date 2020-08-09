use super::Keyword;
use crate::common::*;
use std::fmt::{Display, Write};

#[derive(Clone, Debug, PartialEq)]
pub enum Lexeme {
    /// EOF
    EOF,

    /// CR, LF
    EOL(String),

    /// A keyword e.g. ELSE
    /// The string contains the original text representation (i.e. case sensitive).
    Keyword(Keyword, String),

    /// A sequence of letters (A-Z or a-z) and numbers. The first character is a letter.
    Word(String),

    /// A sequence of whitespace (spaces and tabs)
    Whitespace(String),

    /// A punctuation symbol
    Symbol(char),

    /// An integer number
    Digits(String),
}

pub type LexemeNode = Locatable<Lexeme>;

impl From<LexemeNode> for Lexeme {
    fn from(n: LexemeNode) -> Lexeme {
        let Locatable { element, .. } = n;
        element
    }
}

impl Lexeme {
    pub fn is_eof(&self) -> bool {
        match self {
            Self::EOF => true,
            _ => false,
        }
    }

    pub fn is_eol(&self) -> bool {
        match self {
            Self::EOL(_) => true,
            _ => false,
        }
    }

    pub fn is_eol_or_eof(&self) -> bool {
        match self {
            Self::EOF | Self::EOL(_) => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self, ch: char) -> bool {
        match self {
            Self::Symbol(c) => *c == ch,
            _ => false,
        }
    }

    pub fn is_keyword(&self, keyword: Keyword) -> bool {
        match self {
            Self::Keyword(k, _) => *k == keyword,
            _ => false,
        }
    }

    pub fn is_whitespace(&self) -> bool {
        match self {
            Self::Whitespace(_) => true,
            _ => false,
        }
    }

    pub fn is_word(&self) -> bool {
        match self {
            Self::Word(_) => true,
            _ => false,
        }
    }

    pub fn into_word(self) -> String {
        match self {
            Self::Word(w) => w,
            _ => panic!("Not a word"),
        }
    }

    pub fn into_digits(self) -> String {
        match self {
            Lexeme::Digits(d) => d,
            _ => panic!("Not digits"),
        }
    }
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Keyword(_, s) | Self::Word(s) | Self::Whitespace(s) | Self::Digits(s) => {
                f.write_str(s)
            }
            Lexeme::Symbol(c) => f.write_char(*c),
            Lexeme::EOF | Lexeme::EOL(_) => Err(std::fmt::Error),
        }
    }
}

#[cfg(test)]
impl LexemeNode {
    pub fn word(x: &str, row: u32, col: u32) -> Self {
        Lexeme::Word(x.to_string()).at_rc(row, col)
    }

    pub fn whitespace(row: u32, col: u32) -> Self {
        Lexeme::Whitespace(" ".to_string()).at_rc(row, col)
    }

    pub fn digits(x: &str, row: u32, col: u32) -> Self {
        Lexeme::Digits(x.to_string()).at_rc(row, col)
    }

    pub fn eof(row: u32, col: u32) -> Self {
        Lexeme::EOF.at_rc(row, col)
    }
}
