use super::{HasQualifier, TypeQualifier};
use crate::common::CaseInsensitiveString;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    name: CaseInsensitiveString,
    qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new(name: CaseInsensitiveString, qualifier: TypeQualifier) -> Self {
        QualifiedName { name, qualifier }
    }

    pub fn bare_name(&self) -> &CaseInsensitiveString {
        &self.name
    }
}

impl HasQualifier for QualifiedName {
    fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name, self.qualifier)
    }
}
