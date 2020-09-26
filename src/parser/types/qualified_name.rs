use crate::common::{Locatable, QError};
use crate::parser::types::{BareName, HasQualifier, TypeQualifier};
use std::convert::TryFrom;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    pub bare_name: BareName,
    pub qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new(bare_name: BareName, qualifier: TypeQualifier) -> Self {
        QualifiedName {
            bare_name,
            qualifier,
        }
    }

    pub fn is_of_type(&self, qualifier: TypeQualifier) -> bool {
        self.qualifier == qualifier
    }
}

impl HasQualifier for QualifiedName {
    fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }
}

impl AsRef<BareName> for QualifiedName {
    fn as_ref(&self) -> &BareName {
        &self.bare_name
    }
}

impl From<QualifiedName> for BareName {
    fn from(qualified_name: QualifiedName) -> BareName {
        qualified_name.bare_name
    }
}

impl TryFrom<&str> for QualifiedName {
    type Error = QError;
    fn try_from(s: &str) -> Result<QualifiedName, QError> {
        let mut buf = s.to_owned();
        let last_ch: char = buf.pop().unwrap();
        TypeQualifier::try_from(last_ch).map(|q| QualifiedName::new(BareName::new(buf), q))
    }
}

//
// QualifiedNameNode
//

pub type QualifiedNameNode = Locatable<QualifiedName>;

impl AsRef<BareName> for QualifiedNameNode {
    fn as_ref(&self) -> &BareName {
        let n: &QualifiedName = self.as_ref();
        n.as_ref()
    }
}
