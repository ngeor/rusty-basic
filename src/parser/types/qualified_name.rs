use super::TypeQualifier;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    name: String,
    qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new<S: AsRef<str>>(name: S, qualifier: TypeQualifier) -> QualifiedName {
        QualifiedName {
            name: name.as_ref().to_string(),
            qualifier,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name, self.qualifier)
    }
}
