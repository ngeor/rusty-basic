use crate::common::Locatable;
use crate::parser::types::{BareName, QualifiedName, TypeQualifier};
use crate::parser::{ExpressionType, HasExpressionType};
use std::convert::TryFrom;

#[derive(Clone, Eq, Hash, PartialEq)]
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

    pub fn into_inner(self) -> (BareName, Option<TypeQualifier>) {
        match self {
            Self::Bare(bare_name) => (bare_name, None),
            Self::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) => (bare_name, Some(qualifier)),
        }
    }

    pub fn qualifier(&self) -> Option<TypeQualifier> {
        match self {
            Self::Bare(_) => None,
            Self::Qualified(QualifiedName { qualifier, .. }) => Some(*qualifier),
        }
    }

    pub fn try_concat_name(self, right: Self) -> Option<Self> {
        match self {
            Self::Bare(left_name) => match right {
                Self::Bare(right_bare) => Some(Name::Bare(left_name + '.' + right_bare)),
                Self::Qualified(QualifiedName {
                    bare_name,
                    qualifier,
                }) => Some(Name::Qualified(QualifiedName::new(
                    left_name + '.' + bare_name,
                    qualifier,
                ))),
            },
            _ => None,
        }
    }

    pub fn qualify(&self, qualifier: TypeQualifier) -> Self {
        let bare_name: &BareName = self.as_ref();
        Self::new(bare_name.clone(), Some(qualifier))
    }

    pub fn un_qualify(self) -> Self {
        match self {
            Self::Qualified(QualifiedName { bare_name, .. }) => Self::Bare(bare_name),
            _ => self,
        }
    }

    pub fn demand_bare(self) -> BareName {
        match self {
            Self::Bare(bare_name) => bare_name,
            _ => panic!("{:?} was not bare", self),
        }
    }

    pub fn demand_qualified(self) -> QualifiedName {
        match self {
            Self::Qualified(qualified_name) => qualified_name,
            _ => panic!("{:?} was not qualified", self),
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

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bare(bare_name) => bare_name.fmt(f),
            Self::Qualified(qualified_name) => qualified_name.fmt(f),
        }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl HasExpressionType for Name {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::Bare(_) => ExpressionType::Unresolved,
            Self::Qualified(QualifiedName { qualifier, .. }) => ExpressionType::BuiltIn(*qualifier),
        }
    }
}

//
// NameNode
//

pub type NameNode = Locatable<Name>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NameRef<'a> {
    pub bare_name: &'a BareName,
    pub opt_q: Option<TypeQualifier>,
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
