use super::{HasBareName, QualifiedName, QualifiedNameNode, TypeQualifier};
use crate::common::{AddLocation, CaseInsensitiveString, Locatable, Location};

pub type BareNameNode = Locatable<CaseInsensitiveString>;

impl BareNameNode {
    pub fn to_qualified_name_node(self, qualifier: TypeQualifier) -> QualifiedNameNode {
        self.map_into(|x| QualifiedName::new(x, qualifier))
    }

    pub fn to_qualified_name_node_ref(&self, qualifier: TypeQualifier) -> QualifiedNameNode {
        self.map(|x| QualifiedName::new(x.clone(), qualifier))
    }
}

impl HasBareName for BareNameNode {
    fn bare_name(&self) -> &CaseInsensitiveString {
        self.element()
    }

    fn bare_name_into(self) -> CaseInsensitiveString {
        self.element_into()
    }
}

impl AddLocation<BareNameNode> for CaseInsensitiveString {
    fn add_location(self, pos: Location) -> BareNameNode {
        BareNameNode::new(self, pos)
    }
}

impl PartialEq<CaseInsensitiveString> for BareNameNode {
    fn eq(&self, other: &CaseInsensitiveString) -> bool {
        self.element() == other
    }
}
