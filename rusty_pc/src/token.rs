/// The kind of a token.
/// Possible examples: digits, identifier, keyword, symbol, etc.
pub type TokenKind = u8;

/// Represents a recognized token.
///
/// The `kind` field could have been a generic parameter, but that would require
/// propagating the type too much.
#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    text: String,
}

impl Token {
    pub fn new(kind: TokenKind, text: String) -> Self {
        assert!(!text.is_empty(), "Token text cannot be empty");
        Self { kind, text }
    }

    /// Gets the token kind.
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    /// Gets the token text (owned).
    pub fn to_text(self) -> String {
        self.text
    }

    /// Gets the token text (reference).
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Demands that the token's text consist of a single character
    /// and returns that character. Panics if the token consists of
    /// multiple characters.
    pub fn demand_single_char(&self) -> char {
        self.try_as_single_char().expect("Token is not single char")
    }

    /// If the token's text consists of a single character,
    /// it returns that character.
    /// If the token's text has multiple characters,
    /// it returns `None`.
    pub fn try_as_single_char(&self) -> Option<char> {
        let mut iter = self.text.chars();
        match iter.next() {
            Some(ch) => match iter.next() {
                Some(_) => None,
                None => Some(ch),
            },
            None => None,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_char_token() {
        let token = Token::new(42, "abc".to_string());
        assert_eq!(token.kind(), 42);
        assert_eq!(token.as_str(), "abc");
        assert_eq!(token.try_as_single_char(), None);
        assert_eq!(token.to_string(), "abc");
        assert_eq!(token.to_text(), "abc");
    }

    #[test]
    #[should_panic(expected = "Token is not single char")]
    fn test_multi_char_token_calling_demand_single_char_panics() {
        let token = Token::new(42, "abc".to_string());
        let _ignored = token.demand_single_char();
    }

    #[test]
    fn test_single_char_token() {
        let token = Token::new(19, "a".to_string());
        assert_eq!(token.kind(), 19);
        assert_eq!(token.as_str(), "a");
        assert_eq!(token.demand_single_char(), 'a');
        assert_eq!(token.try_as_single_char(), Some('a'));
        assert_eq!(token.to_string(), "a");
        assert_eq!(token.to_text(), "a");
    }

    #[test]
    #[should_panic(expected = "Token text cannot be empty")]
    fn test_empty_text_panics() {
        let _token = Token::new(42, String::new());
    }
}
