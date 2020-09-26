use crate::common::Locatable;
use crate::parser::types::{BareName, QualifiedName, TypeQualifier};
use std::convert::TryFrom;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Name {
    Bare(BareName),
    Qualified(QualifiedName),
}

impl Name {
    pub fn new(bare_name: BareName, optional_type_qualifier: Option<TypeQualifier>) -> Self {
        match optional_type_qualifier {
            Some(q) => QualifiedName::new(bare_name, q).into(),
            None => bare_name.into(),
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
            Self::Qualified(qualified_name) => qualified_name.is_of_type(qualifier),
        }
    }
}

impl AsRef<BareName> for Name {
    fn as_ref(&self) -> &BareName {
        match self {
            Name::Bare(b) => b,
            Name::Qualified(QualifiedName { bare_name, .. }) => bare_name,
        }
    }
}

impl From<Name> for BareName {
    fn from(n: Name) -> BareName {
        match n {
            Name::Bare(b) => b,
            Name::Qualified(QualifiedName { bare_name, .. }) => bare_name,
        }
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        let mut buf = s.to_string();
        let last_ch: char = buf.pop().unwrap();
        match TypeQualifier::try_from(last_ch) {
            Ok(qualifier) => QualifiedName::new(BareName::new(buf), qualifier).into(),
            _ => {
                buf.push(last_ch);
                BareName::new(buf).into()
            }
        }
    }
}

impl From<BareName> for Name {
    fn from(bare_name: BareName) -> Self {
        Self::Bare(bare_name)
    }
}

impl From<QualifiedName> for Name {
    fn from(qualified_name: QualifiedName) -> Self {
        Self::Qualified(qualified_name)
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
                BareName::new("Pos".to_string()),
                TypeQualifier::PercentInteger
            ))
        );
    }
}
