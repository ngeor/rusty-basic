use crate::common::*;

#[derive(Clone, Debug, PartialEq)]
pub enum LexemeNode {
    /// EOF
    EOF(Location),

    /// CR, LF
    EOL(String, Location),

    /// A sequence of letters (A-Z or a-z)
    Word(String, Location),

    /// A sequence of whitespace (spaces and tabs)
    Whitespace(String, Location),

    /// A punctuation symbol
    Symbol(char, Location),

    /// An integer number
    Digits(u32, Location),
}

impl LexemeNode {
    pub fn push_to(&self, buf: &mut String) {
        match self {
            Self::Word(s, _) | Self::Whitespace(s, _) => buf.push_str(s),
            Self::Symbol(c, _) => buf.push(*c),
            Self::Digits(d, _) => buf.push_str(&format!("{}", d)),
            _ => panic!(format!("Cannot push {:?}", self)),
        }
    }
}

impl HasLocation for LexemeNode {
    fn location(&self) -> Location {
        match self {
            LexemeNode::EOF(pos)
            | LexemeNode::EOL(_, pos)
            | LexemeNode::Word(_, pos)
            | LexemeNode::Whitespace(_, pos)
            | LexemeNode::Symbol(_, pos)
            | LexemeNode::Digits(_, pos) => *pos,
        }
    }
}
