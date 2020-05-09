use super::{HasQualifier, NameTrait, QualifiedName, TypeQualifier};
use crate::common::CaseInsensitiveString;
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub enum Name {
    Bare(CaseInsensitiveString),
    Qualified(QualifiedName),
}

impl Name {
    pub fn new<S: AsRef<str>>(word: S, optional_type_qualifier: Option<TypeQualifier>) -> Self {
        match optional_type_qualifier {
            Some(q) => Self::new_qualified(word, q),
            None => Self::new_bare(word),
        }
    }

    pub fn new_bare<S: AsRef<str>>(word: S) -> Self {
        Name::Bare(CaseInsensitiveString::new(word.as_ref().to_string()))
    }

    pub fn new_qualified<S: AsRef<str>>(word: S, qualifier: TypeQualifier) -> Self {
        Name::Qualified(QualifiedName::new(word, qualifier))
    }
}

impl NameTrait for Name {
    fn bare_name(&self) -> &CaseInsensitiveString {
        match self {
            Self::Bare(b) => b,
            Self::Qualified(t) => t.bare_name(),
        }
    }

    fn opt_qualifier(&self) -> Option<TypeQualifier> {
        match self {
            Self::Bare(_) => None,
            Self::Qualified(t) => Some(t.qualifier()),
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
