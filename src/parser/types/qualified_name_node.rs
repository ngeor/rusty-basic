use super::{QualifiedName, TypeQualifier};
use crate::common::{AddLocation, HasLocation, Location, StripLocation};

#[derive(Clone, Debug, PartialEq)]
pub struct QualifiedNameNode {
    name: String,
    qualifier: TypeQualifier,
    pos: Location,
}

impl QualifiedNameNode {
    pub fn new<S: AsRef<str>>(
        name: S,
        qualifier: TypeQualifier,
        pos: Location,
    ) -> QualifiedNameNode {
        QualifiedNameNode {
            name: name.as_ref().to_string(),
            qualifier,
            pos,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }

    #[cfg(test)]
    pub fn at(&self, location: Location) -> Self {
        QualifiedNameNode::new(&self.name, self.qualifier, location)
    }
}

impl HasLocation for QualifiedNameNode {
    fn location(&self) -> Location {
        self.pos
    }
}

impl AddLocation<QualifiedNameNode> for QualifiedName {
    fn add_location(&self, pos: Location) -> QualifiedNameNode {
        QualifiedNameNode::new(self.name(), self.qualifier(), pos)
    }
}

impl StripLocation<QualifiedName> for QualifiedNameNode {
    fn strip_location(&self) -> QualifiedName {
        QualifiedName::new(&self.name, self.qualifier)
    }
}
