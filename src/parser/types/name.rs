use super::{
    HasBareName, HasQualifier, QualifiedName, ResolvesQualifier, TypeQualifier, TypeResolver,
};
use crate::common::CaseInsensitiveString;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Name {
    Bare(CaseInsensitiveString),
    Typed(QualifiedName),
}

impl Name {
    pub fn to_qualified_name(&self, resolver: &dyn TypeResolver) -> QualifiedName {
        match self {
            Self::Bare(s) => QualifiedName::new(s.clone(), resolver.resolve(s)),
            Self::Typed(t) => t.clone(),
        }
    }

    pub fn to_qualified_name_into(self, resolver: &dyn TypeResolver) -> QualifiedName {
        match self {
            Self::Bare(s) => {
                let qualifier = resolver.resolve(&s);
                QualifiedName::new(s, qualifier)
            }
            Self::Typed(t) => t,
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

impl ResolvesQualifier for Name {
    fn qualifier(&self, resolver: &dyn TypeResolver) -> TypeQualifier {
        match self {
            Self::Bare(b) => resolver.resolve(b),
            Self::Typed(t) => t.qualifier(),
        }
    }
}

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
