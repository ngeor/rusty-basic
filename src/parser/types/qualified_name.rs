use super::{HasQualifier, NameTrait, TypeQualifier};
use crate::common::CaseInsensitiveString;
use std::convert::TryFrom;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    name: CaseInsensitiveString,
    qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new<S: AsRef<str>>(name: S, qualifier: TypeQualifier) -> Self {
        QualifiedName {
            name: CaseInsensitiveString::new(name.as_ref().to_string()),
            qualifier,
        }
    }

    pub fn consume(self) -> (CaseInsensitiveString, TypeQualifier) {
        (self.name, self.qualifier)
    }
}

impl HasQualifier for QualifiedName {
    fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }
}

impl NameTrait for QualifiedName {
    fn bare_name(&self) -> &CaseInsensitiveString {
        &self.name
    }

    fn opt_qualifier(&self) -> Option<TypeQualifier> {
        Some(self.qualifier)
    }
}

impl TryFrom<&str> for QualifiedName {
    type Error = String;
    fn try_from(s: &str) -> Result<QualifiedName, String> {
        let mut buf = s.to_owned();
        let last_ch: char = buf.pop().unwrap();
        TypeQualifier::try_from(last_ch)
            .map(|q| QualifiedName::new(CaseInsensitiveString::new(buf), q))
    }
}
