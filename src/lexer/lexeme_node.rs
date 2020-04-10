use super::Lexeme;
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

impl AddLocation<LexemeNode> for Lexeme {
    fn add_location(&self, pos: Location) -> LexemeNode {
        match self {
            Lexeme::EOF => LexemeNode::EOF(pos),
            Lexeme::EOL(x) => LexemeNode::EOL(x.clone(), pos),
            Lexeme::Word(x) => LexemeNode::Word(x.clone(), pos),
            Lexeme::Whitespace(x) => LexemeNode::Whitespace(x.clone(), pos),
            Lexeme::Symbol(x) => LexemeNode::Symbol(*x, pos),
            Lexeme::Digits(x) => LexemeNode::Digits(*x, pos),
        }
    }
}

impl StripLocation<Lexeme> for LexemeNode {
    fn strip_location(&self) -> Lexeme {
        match self {
            LexemeNode::EOF(_) => Lexeme::EOF,
            LexemeNode::EOL(x, _) => Lexeme::EOL(x.clone()),
            LexemeNode::Word(x, _) => Lexeme::Word(x.clone()),
            LexemeNode::Whitespace(x, _) => Lexeme::Whitespace(x.clone()),
            LexemeNode::Symbol(x, _) => Lexeme::Symbol(*x),
            LexemeNode::Digits(x, _) => Lexeme::Digits(*x),
        }
    }
}
