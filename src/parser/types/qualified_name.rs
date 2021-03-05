use crate::common::{Locatable, QError};
use crate::parser::types::{BareName, TypeQualifier};
use std::convert::TryFrom;

#[derive(Clone, Eq, Hash, PartialEq)]
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

impl AsRef<TypeQualifier> for QualifiedName {
    fn as_ref(&self) -> &TypeQualifier {
        &self.qualifier
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

impl std::fmt::Debug for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.bare_name, f)?;
        std::fmt::Display::fmt(&self.qualifier, f)
    }
}

//
// QualifiedNameNode
//

pub type QualifiedNameNode = Locatable<QualifiedName>;
