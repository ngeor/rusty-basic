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
    pub fn as_word(self) -> Option<String> {
        match self {
            Lexeme::Word(w) => Some(w),
            _ => None,
        }
    }

    // TODO convert in the style of as_digits
    pub fn into_digits(self) -> String {
        match self {
            Lexeme::Digits(d) => d,
            _ => panic!("Not digits"),
        }
    }
}

pub trait LexemeTrait {
    fn is_eof(&self) -> bool;

    fn is_eol_or_eof(&self) -> bool {
        self.is_eof() || self.is_eol()
    }

    fn is_eol(&self) -> bool;

    fn is_symbol(&self, ch: char) -> bool;

    fn is_keyword(&self, keyword: Keyword) -> bool;

    fn is_whitespace(&self) -> bool;

    fn is_word(&self) -> bool;
}

impl LexemeTrait for Lexeme {
    fn is_eof(&self) -> bool {
        self == &Lexeme::EOF
    }

    fn is_eol(&self) -> bool {
        match self {
            Lexeme::EOL(_) => true,
            _ => false,
        }
    }

    fn is_symbol(&self, ch: char) -> bool {
        match self {
            Lexeme::Symbol(c) => *c == ch,
            _ => false,
        }
    }

    fn is_keyword(&self, keyword: Keyword) -> bool {
        match self {
            Lexeme::Keyword(k, _) => *k == keyword,
            _ => false,
        }
    }

    fn is_whitespace(&self) -> bool {
        match self {
            Lexeme::Whitespace(_) => true,
            _ => false,
        }
    }

    fn is_word(&self) -> bool {
        match self {
            Lexeme::Word(_) => true,
            _ => false,
        }
    }
}

impl<T: AsRef<Lexeme>> LexemeTrait for T {
    fn is_eof(&self) -> bool {
        self.as_ref().is_eof()
    }

    fn is_eol(&self) -> bool {
        self.as_ref().is_eol()
    }

    fn is_symbol(&self, ch: char) -> bool {
        self.as_ref().is_symbol(ch)
    }

    fn is_keyword(&self, keyword: Keyword) -> bool {
        self.as_ref().is_keyword(keyword)
    }

    fn is_whitespace(&self) -> bool {
        self.as_ref().is_whitespace()
    }

    fn is_word(&self) -> bool {
        self.as_ref().is_word()
    }
}

impl<T: AsRef<Lexeme>> LexemeTrait for Option<&T> {
    fn is_eof(&self) -> bool {
        match self {
            Some(x) => x.is_eof(),
            None => true, // the only case where we map Option to true
        }
    }
    fn is_eol(&self) -> bool {
        match self {
            Some(x) => x.is_eol(),
            _ => false,
        }
    }

    fn is_symbol(&self, ch: char) -> bool {
        match self {
            Some(x) => x.is_symbol(ch),
            _ => false,
        }
    }

    fn is_keyword(&self, keyword: Keyword) -> bool {
        match self {
            Some(x) => x.is_keyword(keyword),
            _ => false,
        }
    }

    fn is_whitespace(&self) -> bool {
        match self {
            Some(x) => x.is_whitespace(),
            _ => false,
        }
    }

    fn is_word(&self) -> bool {
        match self {
            Some(x) => x.is_word(),
            _ => false,
        }
    }
}

impl HasLocation for Option<LexemeNode> {
    fn pos(&self) -> Location {
        match self {
            Some(x) => x.pos(),
            _ => panic!("None has no location"),
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
