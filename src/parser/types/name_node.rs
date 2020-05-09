use super::Name;
use crate::common::Locatable;

pub type NameNode = Locatable<Name>;

impl PartialEq<Name> for NameNode {
    fn eq(&self, other: &Name) -> bool {
        let my_name: &Name = self.as_ref();
        my_name == other
    }
}

#[cfg(test)]
impl PartialEq<str> for NameNode {
    fn eq(&self, other: &str) -> bool {
        self == &Name::from(other)
    }
}
