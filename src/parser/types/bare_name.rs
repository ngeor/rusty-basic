use crate::common::{CaseInsensitiveString, Locatable};
use crate::parser::pc::{token_list_to_string, Token, TokenList};

pub type BareName = CaseInsensitiveString;
pub type BareNameNode = Locatable<BareName>;

impl AsRef<BareName> for BareName {
    fn as_ref(&self) -> &BareName {
        self
    }
}

impl From<Token> for BareName {
    fn from(token: Token) -> Self {
        Self::new(token.text)
    }
}

impl From<TokenList> for BareName {
    fn from(token_list: TokenList) -> Self {
        Self::new(token_list_to_string(token_list))
    }
}
