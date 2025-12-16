use crate::BareName;
use rusty_common::Position;

pub type TokenKind = u8;

// TODO make fields private
// TODO remove the Clone trait
/// Represents a recognized token.
///
/// The [kind] field could have been a generic parameter, but that would require
/// propagating the type in the [Tokenizer] and eventually also to the parsers.
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub pos: Position,
}

pub type TokenList = Vec<Token>;

pub fn token_list_to_string(tokens: TokenList) -> String {
    tokens.into_iter().map(|token| token.text).collect()
}

pub fn token_list_to_bare_name(tokens: TokenList) -> BareName {
    BareName::new(token_list_to_string(tokens))
}

pub trait Tokenizer {
    // TODO this can also be Result<Token, ?> where ? is Fatal/NotFound, or an Iterator
    #[deprecated]
    fn read(&mut self) -> Option<Token>;
    #[deprecated]
    fn unread(&mut self);
    #[deprecated]
    fn position(&self) -> Position;
}
