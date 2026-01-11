/// The kind of a token.
/// Possible examples: digits, identifier, keyword, symbol, etc.
pub type TokenKind = u8;

/// Represents a recognized token.
///
/// The [kind] field could have been a generic parameter, but that would require
/// propagating the type too much.
#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    text: String,
}

impl Token {
    pub fn new(kind: TokenKind, text: String) -> Self {
        Self { kind, text }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn text(self) -> String {
        self.text
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn as_char(&self) -> char {
        self.text.chars().next().unwrap()
    }

    pub fn len(&self) -> usize {
        self.text.chars().count()
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}
