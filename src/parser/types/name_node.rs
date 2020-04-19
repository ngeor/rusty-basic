use super::{HasBareName, Name, QualifiedName, ResolvesQualifier, TypeQualifier, TypeResolver};
use crate::common::{CaseInsensitiveString, Locatable, Location};

pub type NameNode = Locatable<Name>;
pub type BareNameNode = Locatable<CaseInsensitiveString>;

impl NameNode {
    pub fn from(
        word: String,
        optional_type_qualifier: Option<TypeQualifier>,
        pos: Location,
    ) -> Self {
        let s = CaseInsensitiveString::new(word);
        let n = match optional_type_qualifier {
            Some(q) => Name::Typed(QualifiedName::new(s, q)),
            None => Name::Bare(s),
        };
        NameNode::new(n, pos)
    }
}

impl HasBareName for NameNode {
    fn bare_name(&self) -> &CaseInsensitiveString {
        self.element().bare_name()
    }

    fn bare_name_into(self) -> CaseInsensitiveString {
        self.element_into().bare_name_into()
    }
}

impl ResolvesQualifier for NameNode {
    fn qualifier<T: TypeResolver>(&self, resolver: &T) -> TypeQualifier {
        self.element().qualifier(resolver)
    }
}

impl PartialEq<Name> for NameNode {
    fn eq(&self, other: &Name) -> bool {
        self.element() == other
    }
}

#[cfg(test)]
impl PartialEq<str> for NameNode {
    fn eq(&self, other: &str) -> bool {
        self == &Name::from(other)
    }
}

#[cfg(test)]
impl PartialEq<str> for BareNameNode {
    fn eq(&self, other: &str) -> bool {
        self.element() == other
    }
}
