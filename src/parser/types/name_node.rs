use super::{BareNameNode, Name, QualifiedNameNode, TypeQualifier, TypeResolver};
use crate::common::{AddLocation, HasLocation, Location, StripLocation};

#[derive(Clone, Debug, PartialEq)]
pub enum NameNode {
    Bare(BareNameNode),
    Typed(QualifiedNameNode),
}

impl NameNode {
    pub fn new<S: AsRef<str>>(
        name: S,
        opt_qualifier: Option<TypeQualifier>,
        pos: Location,
    ) -> NameNode {
        match opt_qualifier {
            Some(qualifier) => NameNode::Typed(QualifiedNameNode::new(name, qualifier, pos)),
            _ => NameNode::Bare(BareNameNode::new(name, pos)),
        }
    }

    pub fn name(&self) -> &String {
        match self {
            NameNode::Bare(b) => b.name(),
            NameNode::Typed(t) => t.name(),
        }
    }

    pub fn resolve(&self, resolver: &dyn TypeResolver) -> QualifiedNameNode {
        match self {
            NameNode::Bare(b) => b.to_qualified_name_node(resolver.resolve(b.name())),
            NameNode::Typed(t) => t.clone(),
        }
    }

    #[cfg(test)]
    pub fn at(&self, location: Location) -> Self {
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

impl HasLocation for NameNode {
    fn location(&self) -> Location {
        match self {
            NameNode::Bare(n) => n.location(),
            NameNode::Typed(n) => n.location(),
        }
    }
}

impl AddLocation<NameNode> for Name {
    fn add_location(&self, pos: Location) -> NameNode {
        match self {
            Name::Bare(b) => NameNode::Bare(b.add_location(pos)),
            Name::Typed(t) => NameNode::Typed(t.add_location(pos)),
        }
    }
}

impl StripLocation<Name> for NameNode {
    fn strip_location(&self) -> Name {
        match self {
            NameNode::Bare(b) => Name::Bare(b.strip_location()),
            NameNode::Typed(t) => Name::Typed(t.strip_location()),
        }
    }
}
