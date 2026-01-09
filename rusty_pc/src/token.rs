pub type TokenKind = u8;

// TODO remove the Clone trait
/// Represents a recognized token.
///
/// The [kind] field could have been a generic parameter, but that would require
/// propagating the type in the [Tokenizer] and eventually also to the parsers.
#[derive(Clone, Debug)]
pub struct Token {
    kind: TokenKind,
    // TODO char or String text
    text: String,
}

impl Token {
    pub fn new(kind: TokenKind, text: String) -> Self {
        Self { kind, text }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn to_str(self) -> String {
        self.text
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }
}

pub type TokenList = Vec<Token>;

// TODO move elsewhere or deprecate
pub fn token_list_to_string(tokens: TokenList) -> String {
    tokens.into_iter().map(|token| token.text).collect()
}
