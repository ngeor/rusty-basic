use super::{HasBareName, HasQualifier, QualifiedName, TypeQualifier};
use crate::common::{AddLocation, CaseInsensitiveString, Locatable, Location};

pub type QualifiedNameNode = Locatable<QualifiedName>;

impl HasBareName for QualifiedNameNode {
    fn bare_name(&self) -> &CaseInsensitiveString {
        self.element().bare_name()
    }

    fn bare_name_into(self) -> CaseInsensitiveString {
        self.element_into().bare_name_into()
    }
}

impl HasQualifier for QualifiedNameNode {
    fn qualifier(&self) -> TypeQualifier {
        self.element().qualifier()
    }
}

impl AddLocation<QualifiedNameNode> for QualifiedName {
    fn add_location(self, pos: Location) -> QualifiedNameNode {
        QualifiedNameNode::new(self, pos)
    }
}

impl PartialEq<QualifiedName> for QualifiedNameNode {
    fn eq(&self, other: &QualifiedName) -> bool {
        self.element() == other
    }
}
