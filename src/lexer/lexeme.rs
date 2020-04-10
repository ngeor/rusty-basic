#[derive(Clone, Debug, PartialEq)]
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
            Self::Word(s) | Self::Whitespace(s) => buf.push_str(s),
            Self::Symbol(c) => buf.push(*c),
            Self::Digits(d) => buf.push_str(&format!("{}", d)),
            _ => panic!(format!("Cannot push {:?}", self)),
        }
    }
}
