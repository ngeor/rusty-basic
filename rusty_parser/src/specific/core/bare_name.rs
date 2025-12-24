use crate::pc::Token;
use rusty_common::{CaseInsensitiveString, Positioned};

pub type BareName = CaseInsensitiveString;
pub type BareNamePos = Positioned<BareName>;

impl From<Token> for BareName {
    fn from(token: Token) -> Self {
        Self::new(token.text)
    }
}
