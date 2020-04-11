use super::{HasBareName, QualifiedName};
use crate::common::CaseInsensitiveString;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Name {
    Bare(CaseInsensitiveString),
    Typed(QualifiedName),
}

impl HasBareName for Name {
    fn bare_name(&self) -> &CaseInsensitiveString {
        match self {
            Self::Bare(x) => x,
            Self::Typed(t) => t.bare_name(),
        }
    }

    fn bare_name_into(self) -> CaseInsensitiveString {
        match self {
            Self::Bare(x) => x,
            Self::Typed(t) => t.bare_name_into(),
        }
    }
}

#[cfg(test)]
use super::TypeQualifier;
#[cfg(test)]
use std::convert::TryFrom;

#[cfg(test)]
impl Name {
    pub fn from<S: AsRef<str>>(s: S) -> Self {
        let mut buf = s.as_ref().to_string();
        let last_ch: char = buf.pop().unwrap();
        match TypeQualifier::try_from(last_ch) {
            Ok(qualifier) => Name::Typed(QualifiedName::new(
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

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Name::Bare(s) => write!(f, "{}", s),
            Name::Typed(t) => write!(f, "{}", t),
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
            Name::Typed(QualifiedName::new(
                CaseInsensitiveString::new("Pos".to_string()),
                TypeQualifier::PercentInteger
            ))
        );
    }
}
