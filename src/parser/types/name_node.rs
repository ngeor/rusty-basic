use super::{
    BareNameNode, HasBareName, HasQualifier, Name, QualifiedName, QualifiedNameNode, TypeQualifier,
    TypeResolver,
};
use crate::common::{AddLocation, CaseInsensitiveString, HasLocation, Location, StripLocation};

#[derive(Clone, Debug, PartialEq)]
pub enum NameNode {
    Bare(BareNameNode),
    Typed(QualifiedNameNode),
}

impl NameNode {
    pub fn new(
        name: CaseInsensitiveString,
        opt_qualifier: Option<TypeQualifier>,
        pos: Location,
    ) -> NameNode {
        match opt_qualifier {
            Some(qualifier) => NameNode::Typed(QualifiedNameNode::new(
                QualifiedName::new(name, qualifier),
                pos,
            )),
            _ => NameNode::Bare(BareNameNode::new(name, pos)),
        }
    }

    pub fn resolve(self, resolver: &dyn TypeResolver) -> QualifiedNameNode {
        match self {
            NameNode::Bare(b) => {
                let qualifier = resolver.resolve(b.element());
                b.to_qualified_name_node(qualifier)
            }
            NameNode::Typed(t) => t,
        }
    }

    pub fn resolve_qualifier(&self, resolver: &dyn TypeResolver) -> TypeQualifier {
        match self {
            NameNode::Bare(b) => resolver.resolve(b.element()),
            NameNode::Typed(t) => t.qualifier(),
        }
    }

    pub fn resolve_ref(&self, resolver: &dyn TypeResolver) -> QualifiedNameNode {
        match self {
            NameNode::Bare(b) => {
                let qualifier = resolver.resolve(b.element());
                b.to_qualified_name_node_ref(qualifier)
            }
            NameNode::Typed(t) => t.clone(),
        }
    }

    #[cfg(test)]
    pub fn at(self, location: Location) -> Self {
        match self {
            NameNode::Bare(b) => NameNode::Bare(b.at(location)),
            NameNode::Typed(t) => NameNode::Typed(t.at(location)),
        }
    }

    #[cfg(test)]
    pub fn from<S: AsRef<str>>(s: S) -> Self {
        Name::from(s).add_location(Location::zero())
    }
}

impl HasBareName for NameNode {
    fn bare_name(&self) -> &CaseInsensitiveString {
        match self {
            NameNode::Bare(b) => b.bare_name(),
            NameNode::Typed(t) => t.bare_name(),
        }
    }

    fn bare_name_into(self) -> CaseInsensitiveString {
        match self {
            NameNode::Bare(b) => b.bare_name_into(),
            NameNode::Typed(t) => t.bare_name_into(),
        }
    }
}

impl HasLocation for NameNode {
    fn location(&self) -> Location {
        match self {
            NameNode::Bare(n) => n.location(),
            NameNode::Typed(n) => n.location(),
        }
    }
}

impl AddLocation<NameNode> for Name {
    fn add_location(self, pos: Location) -> NameNode {
        match self {
            Name::Bare(b) => NameNode::Bare(b.add_location(pos)),
            Name::Typed(t) => NameNode::Typed(t.add_location(pos)),
        }
    }
}

impl StripLocation<Name> for NameNode {
    fn strip_location(self) -> Name {
        match self {
            NameNode::Bare(b) => Name::Bare(b.strip_location()),
            NameNode::Typed(t) => Name::Typed(t.strip_location()),
        }
    }
}
