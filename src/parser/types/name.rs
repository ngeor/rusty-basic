use super::{HasQualifier, TypeQualifier};
use crate::common::{CaseInsensitiveString, Locatable, QError};
use std::convert::TryFrom;

//
// BareName
//

pub type BareName = CaseInsensitiveString;

// QualifiedName -> BareName

impl From<QualifiedName> for BareName {
    fn from(qualified_name: QualifiedName) -> BareName {
        qualified_name.bare_name
    }
}

// &QualifiedName -> BareName

impl From<&QualifiedName> for BareName {
    fn from(qualified_name: &QualifiedName) -> BareName {
        let b: &BareName = qualified_name.as_ref();
        b.clone()
    }
}

// Name -> BareName

impl From<Name> for BareName {
    fn from(n: Name) -> BareName {
        match n {
            Name::Bare(b) => b,
            Name::Qualified {
                bare_name: name, ..
            } => name,
        }
    }
}

// &QualifiedNameNode -> BareName

impl From<&QualifiedNameNode> for BareName {
    fn from(n: &QualifiedNameNode) -> BareName {
        let name: &QualifiedName = n.as_ref();
        name.clone().into()
    }
}

// &NameNode -> BareName

impl From<&NameNode> for BareName {
    fn from(n: &NameNode) -> BareName {
        let name: &Name = n.as_ref();
        name.clone().into()
    }
}

// NameNode -> BareName

impl From<NameNode> for BareName {
    fn from(n: NameNode) -> BareName {
        let NameNode { element, .. } = n;
        element.into()
    }
}

// AsRef<BareName>

impl AsRef<BareName> for BareName {
    fn as_ref(&self) -> &BareName {
        self
    }
}

//
// BareNameNode
//

pub type BareNameNode = Locatable<BareName>;

//
// QualifiedName
//

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    pub bare_name: CaseInsensitiveString,
    pub qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new(bare_name: CaseInsensitiveString, qualifier: TypeQualifier) -> Self {
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

impl TryFrom<&str> for QualifiedName {
    type Error = QError;
    fn try_from(s: &str) -> Result<QualifiedName, QError> {
        let mut buf = s.to_owned();
        let last_ch: char = buf.pop().unwrap();
        TypeQualifier::try_from(last_ch)
            .map(|q| QualifiedName::new(CaseInsensitiveString::new(buf), q))
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

//
// Name
//

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Name {
    Bare(CaseInsensitiveString),
    // TODO put back the QualifiedName here
    Qualified {
        bare_name: CaseInsensitiveString,
        qualifier: TypeQualifier,
    },
}

impl Name {
    pub fn new(
        bare_name: CaseInsensitiveString,
        optional_type_qualifier: Option<TypeQualifier>,
    ) -> Self {
        match optional_type_qualifier {
            Some(q) => Self::new_qualified(bare_name, q),
            None => Self::new_bare(bare_name),
        }
    }

    pub fn new_bare(bare_name: CaseInsensitiveString) -> Self {
        Name::Bare(bare_name)
    }

    pub fn new_qualified(bare_name: CaseInsensitiveString, qualifier: TypeQualifier) -> Self {
        Name::Qualified {
            bare_name,
            qualifier,
        }
    }

    pub fn is_bare(&self) -> bool {
        match self {
            Self::Bare(_) => true,
            _ => false,
        }
    }

    pub fn is_bare_or_of_type(&self, qualifier: TypeQualifier) -> bool {
        match self {
            Self::Bare(_) => true,
            Self::Qualified { qualifier: q, .. } => *q == qualifier,
        }
    }
}

impl AsRef<BareName> for Name {
    fn as_ref(&self) -> &BareName {
        match self {
            Name::Bare(b) => b,
            Name::Qualified {
                bare_name: name, ..
            } => name,
        }
    }
}

impl<S: AsRef<str>> From<S> for Name {
    fn from(s: S) -> Self {
        let mut buf = s.as_ref().to_string();
        let last_ch: char = buf.pop().unwrap();
        match TypeQualifier::try_from(last_ch) {
            Ok(qualifier) => Name::Qualified {
                bare_name: CaseInsensitiveString::new(buf),
                qualifier,
            },
            _ => {
                buf.push(last_ch);
                Name::Bare(CaseInsensitiveString::new(buf))
            }
        }
    }
}

impl From<QualifiedName> for Name {
    fn from(qualified_name: QualifiedName) -> Self {
        let QualifiedName {
            bare_name,
            qualifier,
        } = qualified_name;
        Self::Qualified {
            bare_name,
            qualifier,
        }
    }
}

//
// NameNode
//

pub type NameNode = Locatable<Name>;

impl AsRef<BareName> for NameNode {
    fn as_ref(&self) -> &BareName {
        let n: &Name = self.as_ref();
        n.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(Name::from("A"), Name::Bare("A".into()));
        assert_eq!(
            Name::from("Pos%"),
            Name::Qualified {
                bare_name: CaseInsensitiveString::new("Pos".to_string()),
                qualifier: TypeQualifier::PercentInteger
            }
        );
    }
}
