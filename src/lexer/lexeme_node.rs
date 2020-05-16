use super::Keyword;
use crate::common::Location;

#[derive(Clone, Debug, PartialEq)]
pub enum LexemeNode {
    /// EOF
    EOF(Location),

    /// CR, LF
    EOL(String, Location),

    /// A keyword e.g. ELSE
    /// The string contains the original text representation (i.e. case sensitive).
    Keyword(Keyword, String, Location),

    /// A sequence of letters (A-Z or a-z)
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
}
