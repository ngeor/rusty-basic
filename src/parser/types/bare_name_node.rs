use crate::common::{CaseInsensitiveString, Locatable};

pub type BareNameNode = Locatable<CaseInsensitiveString>;

impl From<BareNameNode> for CaseInsensitiveString {
    fn from(n: BareNameNode) -> CaseInsensitiveString {
        n.consume().0
    }
}

#[cfg(test)]
impl PartialEq<str> for BareNameNode {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}
