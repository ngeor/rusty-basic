use super::Keyword;
use crate::common::{HasLocation, Location};

#[derive(Clone, Debug, PartialEq)]
pub enum LexemeNode {
    /// EOF
    EOF(Location),

    /// CR, LF
    EOL(String, Location),

    /// A keyword e.g. ELSE
    /// The string contains the original text representation (i.e. case sensitive).
    Keyword(Keyword, String, Location),

    /// A sequence of letters (A-Z or a-z) and numbers. The first character is a letter.
    Word(String, Location),

    /// A sequence of whitespace (spaces and tabs)
    Whitespace(String, Location),

    /// A punctuation symbol
    Symbol(char, Location),

    /// An integer number
    Digits(String, Location),
}

impl LexemeNode {
    pub fn is_eof(&self) -> bool {
        match self {
            LexemeNode::EOF(_) => true,
            _ => false,
        }
    }

    pub fn is_eol(&self) -> bool {
        match self {
            LexemeNode::EOL(_, _) => true,
            _ => false,
        }
    }

    pub fn is_eol_or_eof(&self) -> bool {
        match self {
            LexemeNode::EOF(_) | LexemeNode::EOL(_, _) => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self, ch: char) -> bool {
        match self {
            LexemeNode::Symbol(c, _) => *c == ch,
            _ => false,
        }
    }

    pub fn is_keyword(&self, keyword: Keyword) -> bool {
        match self {
            LexemeNode::Keyword(k, _, _) => *k == keyword,
            _ => false,
        }
    }

    pub fn is_whitespace(&self) -> bool {
        match self {
            LexemeNode::Whitespace(_, _) => true,
            _ => false,
        }
    }

    pub fn is_word(&self) -> bool {
        match self {
            LexemeNode::Word(_, _) => true,
            _ => false,
        }
    }

    pub fn into_word(self) -> (String, Location) {
        match self {
            LexemeNode::Word(w, pos) => (w, pos),
            _ => panic!("Not a word"),
        }
    }

    pub fn into_digits(self) -> (String, Location) {
        match self {
            LexemeNode::Digits(d, pos) => (d, pos),
            _ => panic!("Not digits"),
        }
    }
}

impl HasLocation for LexemeNode {
    fn pos(&self) -> Location {
        match self {
            Self::EOF(pos)
            | Self::EOL(_, pos)
            | Self::Keyword(_, _, pos)
            | Self::Word(_, pos)
            | Self::Whitespace(_, pos)
            | Self::Symbol(_, pos)
            | Self::Digits(_, pos) => pos.clone(),
        }
    }
}

#[cfg(test)]
impl LexemeNode {
    pub fn word(x: &str, row: u32, col: u32) -> Self {
        Self::Word(x.to_string(), Location::new(row, col))
    }

    pub fn whitespace(row: u32, col: u32) -> Self {
        Self::Whitespace(" ".to_string(), Location::new(row, col))
    }

    pub fn digits(x: &str, row: u32, col: u32) -> Self {
        Self::Digits(x.to_string(), Location::new(row, col))
    }

    pub fn eof(row: u32, col: u32) -> Self {
        Self::EOF(Location::new(row, col))
    }
}
