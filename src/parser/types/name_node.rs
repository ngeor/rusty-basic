use super::{Name, QualifiedName, ResolveInto, ResolveIntoRef, TypeQualifier, TypeResolver};
use crate::common::{CaseInsensitiveString, Locatable, Location};

pub type NameNode = Locatable<Name>;

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

impl AsRef<CaseInsensitiveString> for NameNode {
    fn as_ref(&self) -> &CaseInsensitiveString {
        let name: &Name = self.as_ref();
        name.as_ref()
    }
}

impl ResolveIntoRef<TypeQualifier> for NameNode {
    fn resolve_into<T: TypeResolver>(&self, resolver: &T) -> TypeQualifier {
        let name: &Name = self.as_ref();
        name.resolve_into(resolver)
    }
}

impl ResolveIntoRef<QualifiedName> for NameNode {
    fn resolve_into<T: TypeResolver>(&self, resolver: &T) -> QualifiedName {
        let name: &Name = self.as_ref();
        name.resolve_into(resolver)
    }
}

impl ResolveInto<QualifiedName> for NameNode {
    fn resolve_into<T: TypeResolver>(self, resolver: &T) -> QualifiedName {
        let name: Name = self.into();
        name.resolve_into(resolver)
    }
}

impl PartialEq<Name> for NameNode {
    fn eq(&self, other: &Name) -> bool {
        let my_name: &Name = self.as_ref();
        my_name == other
    }
}

impl From<NameNode> for Name {
    fn from(n: NameNode) -> Name {
        n.consume().0
    }
}

#[cfg(test)]
impl PartialEq<str> for NameNode {
    fn eq(&self, other: &str) -> bool {
        self == &Name::from(other)
    }
}
