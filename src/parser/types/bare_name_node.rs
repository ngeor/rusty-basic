use super::{QualifiedNameNode, TypeQualifier};
use crate::common::{AddLocation, HasLocation, Location, StripLocation, StripLocationRef};

#[derive(Clone, Debug, PartialEq)]
pub struct BareNameNode {
    name: String,
    pos: Location,
}

impl BareNameNode {
    pub fn new<S: AsRef<str>>(name: S, pos: Location) -> BareNameNode {
        BareNameNode {
            name: name.as_ref().to_string(),
            pos,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn to_qualified_name_node(&self, qualifier: TypeQualifier) -> QualifiedNameNode {
        QualifiedNameNode::new(&self.name, qualifier, self.pos)
    }

    #[cfg(test)]
    pub fn at(&self, location: Location) -> Self {
        BareNameNode::new(&self.name, location)
    }
}

impl HasLocation for BareNameNode {
    fn location(&self) -> Location {
        self.pos
    }
}

impl AddLocation<BareNameNode> for String {
    fn add_location(&self, pos: Location) -> BareNameNode {
        BareNameNode::new(self, pos)
    }
}

impl StripLocationRef<String> for BareNameNode {
    fn strip_location_ref(&self) -> &String {
        &self.name
    }
}

impl StripLocation<String> for BareNameNode {
    fn strip_location(&self) -> String {
        self.name.clone()
    }
}
