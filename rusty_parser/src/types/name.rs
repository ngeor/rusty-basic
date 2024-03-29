use crate::types::{BareName, ExpressionType, HasExpressionType, QualifiedName, TypeQualifier};
use rusty_common::Positioned;
#[cfg(test)]
use std::convert::TryFrom;

/// Defines a name.
///
/// Parsing syntax reference
///
/// ```txt
/// <qualifier> ::= "!" | "#" | "$" | "%" | "&"
///
/// <bare-name-with-dots-not-keyword> ::= <bare-name-with-dots> AND NOT keyword
/// <bare-name-with-dots> ::= <letter> | <letter><letters-or-digits-or-dots>
///
/// <bare-name-not-keyword> ::= <bare-name> AND NOT keyword
/// <bare-name> ::= <letter> | <letter><letters-or-digits>
///
/// <letters-or-digits-or-dots> ::= <letter-or-digit-or-dot> | <letter-or-digit-or-dot><letters-or-digits-or-dots>
/// <letter-or-digit-or-dot> ::= <letter> | <digit> | "."
///
/// <letters-or-digits> ::= <letter-or-digit> | <letter-or-digit><letters-or-digits>
/// <letter-or-digit> ::= <letter> | <digit>
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Name {
    Bare(BareName),
    Qualified(BareName, TypeQualifier),
}

impl Name {
    pub fn new(bare_name: BareName, optional_type_qualifier: Option<TypeQualifier>) -> Self {
        match optional_type_qualifier {
            Some(q) => Self::Qualified(bare_name, q),
            None => Self::Bare(bare_name),
        }
    }

    pub fn is_bare(&self) -> bool {
        matches!(self, Self::Bare(_))
    }

    pub fn is_bare_or_of_type(&self, qualifier: TypeQualifier) -> bool {
        match self {
            Self::Bare(_) => true,
            Self::Qualified(_, q) => *q == qualifier,
        }
    }

    pub fn qualifier(&self) -> Option<TypeQualifier> {
        match self {
            Self::Bare(_) => None,
            Self::Qualified(_, qualifier) => Some(*qualifier),
        }
    }

    pub fn try_concat_name(self, right: Self) -> Option<Self> {
        match self {
            Self::Bare(left_name) => match right {
                Self::Bare(right_bare) => Some(Name::Bare(Self::dot_concat(left_name, right_bare))),
                Self::Qualified(right_bare, qualifier) => Some(Name::Qualified(
                    Self::dot_concat(left_name, right_bare),
                    qualifier,
                )),
            },
            _ => None,
        }
    }

    pub fn dot_concat(mut left: BareName, right: BareName) -> BareName {
        left.push('.');
        left.push_str(right.as_ref());
        left
    }

    pub fn demand_bare(self) -> BareName {
        match self {
            Self::Bare(bare_name) => bare_name,
            _ => panic!("{:?} was not bare", self),
        }
    }

    pub fn demand_qualified(self) -> QualifiedName {
        match self {
            Self::Qualified(bare_name, qualifier) => QualifiedName::new(bare_name, qualifier),
            _ => panic!("{:?} was not qualified", self),
        }
    }

    pub fn bare_name(&self) -> &BareName {
        match self {
            Name::Bare(bare_name) | Name::Qualified(bare_name, _) => bare_name,
        }
    }
}

impl From<Name> for BareName {
    fn from(name: Name) -> Self {
        match name {
            Name::Bare(bare_name) | Name::Qualified(bare_name, _) => bare_name,
        }
    }
}

// TODO #[cfg(test)]
impl From<&str> for Name {
    fn from(s: &str) -> Self {
        let mut buf = s.to_string();
        let last_ch: char = buf.pop().unwrap();
        match TypeQualifier::try_from(last_ch) {
            Ok(qualifier) => Self::Qualified(buf.into(), qualifier),
            _ => {
                buf.push(last_ch);
                Self::Bare(buf.into())
            }
        }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bare(bare_name) => bare_name.fmt(f),
            Self::Qualified(bare_name, qualifier) => {
                bare_name.fmt(f).and_then(|_| qualifier.fmt(f))
            }
        }
    }
}

impl HasExpressionType for Name {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::Bare(_) => ExpressionType::Unresolved,
            Self::Qualified(_, qualifier) => ExpressionType::BuiltIn(*qualifier),
        }
    }
}

/// A [Name] with position information.
pub type NamePos = Positioned<Name>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(Name::from("A"), Name::Bare("A".into()));
        assert_eq!(
            Name::from("Pos%"),
            Name::Qualified(
                BareName::new("Pos".to_string()),
                TypeQualifier::PercentInteger
            )
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Name::from("Foo").to_string(), "Foo");
        assert_eq!(Name::from("age%").to_string(), "age%");
    }
}
