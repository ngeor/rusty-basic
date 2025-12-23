use crate::specific::{BareName, Name, TypeQualifier};
use rusty_common::*;
// TODO #[cfg(test)]
use crate::error::ParseError;
use std::convert::TryFrom;

/// A qualified name is a bare name followed by a built-in type qualifier.
/// Example: `name$`, `age%`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    bare_name: BareName,
    qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new(bare_name: BareName, qualifier: TypeQualifier) -> Self {
        QualifiedName {
            bare_name,
            qualifier,
        }
    }

    pub fn into_inner(self) -> (BareName, TypeQualifier) {
        (self.bare_name, self.qualifier)
    }
}

impl From<QualifiedName> for Name {
    fn from(qualified_name: QualifiedName) -> Self {
        Self::Qualified(qualified_name.bare_name, qualified_name.qualifier)
    }
}

// TODO #[cfg(test)]
impl TryFrom<&str> for QualifiedName {
    type Error = ParseError;
    fn try_from(s: &str) -> Result<QualifiedName, ParseError> {
        let mut buf = s.to_owned();
        let last_ch: char = buf.pop().unwrap();
        TypeQualifier::try_from(last_ch).map(|q| QualifiedName::new(BareName::new(buf), q))
    }
}

impl std::fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.bare_name, f)?;
        std::fmt::Display::fmt(&self.qualifier, f)
    }
}

/// A [QualifiedName] with position information.
pub type QualifiedNamePos = Positioned<QualifiedName>;
