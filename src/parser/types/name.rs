use super::{HasQualifier, TypeQualifier};
use crate::common::{CaseInsensitiveString, Locatable};
use std::convert::TryFrom;

//
// WithTypeQualifier
//

pub trait WithTypeQualifier {
    fn with_type(self, q: TypeQualifier) -> QualifiedName;
    fn with_type_ref(&self, q: TypeQualifier) -> QualifiedName;
}

//
// BareName
//

pub type BareName = CaseInsensitiveString;

// QualifiedName -> BareName

impl From<QualifiedName> for BareName {
    fn from(n: QualifiedName) -> BareName {
        n.name
    }
}

// &QualifiedName -> BareName

impl From<&QualifiedName> for BareName {
    fn from(n: &QualifiedName) -> BareName {
        let b: &BareName = n.as_ref();
        b.clone()
    }
}

// Name -> BareName

impl From<Name> for BareName {
    fn from(n: Name) -> BareName {
        match n {
            Name::Bare(b) => b,
            Name::Qualified(q) => q.into(),
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

// WithTypeQualifier

impl WithTypeQualifier for BareName {
    fn with_type(self, q: TypeQualifier) -> QualifiedName {
        QualifiedName::new(self, q)
    }

    fn with_type_ref(&self, q: TypeQualifier) -> QualifiedName {
        self.clone().with_type(q)
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
    pub name: CaseInsensitiveString,
    pub qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new(name: CaseInsensitiveString, qualifier: TypeQualifier) -> Self {
        QualifiedName { name, qualifier }
    }

    pub fn is_of_type(&self, q_other: TypeQualifier) -> bool {
        self.qualifier == q_other
    }
}

impl HasQualifier for QualifiedName {
    fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }
}

impl AsRef<BareName> for QualifiedName {
    fn as_ref(&self) -> &BareName {
        &self.name
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

impl WithTypeQualifier for QualifiedName {
    fn with_type(self, q: TypeQualifier) -> QualifiedName {
        QualifiedName::new(self.name, q)
    }

    fn with_type_ref(&self, q: TypeQualifier) -> QualifiedName {
        self.clone().with_type(q)
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

#[derive(Clone, Debug, PartialEq)]
pub enum Name {
    Bare(CaseInsensitiveString),
    Qualified(QualifiedName),
}

impl Name {
    pub fn new(
        word: CaseInsensitiveString,
        optional_type_qualifier: Option<TypeQualifier>,
    ) -> Self {
        match optional_type_qualifier {
            Some(q) => Self::new_qualified(word, q),
            None => Self::new_bare(word),
        }
    }

    pub fn new_bare(word: CaseInsensitiveString) -> Self {
        Name::Bare(word)
    }

    pub fn new_qualified(word: CaseInsensitiveString, qualifier: TypeQualifier) -> Self {
        Name::Qualified(QualifiedName::new(word, qualifier))
    }

    pub fn is_bare(&self) -> bool {
        match self {
            Self::Bare(_) => true,
            _ => false,
        }
    }

    pub fn is_bare_or_of_type(&self, q_other: TypeQualifier) -> bool {
        match self {
            Self::Bare(_) => true,
            Self::Qualified(q) => q.is_of_type(q_other),
        }
    }
}

impl WithTypeQualifier for Name {
    fn with_type(self, q: TypeQualifier) -> QualifiedName {
        match self {
            Self::Bare(b) => b.with_type(q),
            Self::Qualified(q_name) => q_name.with_type(q),
        }
    }

    fn with_type_ref(&self, q: TypeQualifier) -> QualifiedName {
        self.clone().with_type(q)
    }
}

impl AsRef<BareName> for Name {
    fn as_ref(&self) -> &BareName {
        match self {
            Name::Bare(b) => b,
            Name::Qualified(q) => q.as_ref(),
        }
    }
}

impl<S: AsRef<str>> From<S> for Name {
    fn from(s: S) -> Self {
        let mut buf = s.as_ref().to_string();
        let last_ch: char = buf.pop().unwrap();
        match TypeQualifier::try_from(last_ch) {
            Ok(qualifier) => Name::Qualified(QualifiedName::new(
                CaseInsensitiveString::new(buf),
                qualifier,
            )),
            _ => {
                buf.push(last_ch);
                Name::Bare(CaseInsensitiveString::new(buf))
            }
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
            Name::Qualified(QualifiedName::new(
                CaseInsensitiveString::new("Pos".to_string()),
                TypeQualifier::PercentInteger
            ))
        );
    }
}
