use crate::parser::pc::Token;
use rusty_common::{CaseInsensitiveString, Locatable};

pub type BareName = CaseInsensitiveString;
pub type BareNameNode = Locatable<BareName>;

impl From<Token> for BareName {
    fn from(token: Token) -> Self {
        Self::new(token.text)
    }
}
