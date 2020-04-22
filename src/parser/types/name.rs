use super::{
    HasQualifier, QualifiedName, ResolveInto, ResolveIntoRef, TypeQualifier, TypeResolver,
};
use crate::common::CaseInsensitiveString;
use std::convert::TryFrom;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Name {
    Bare(CaseInsensitiveString),
    Typed(QualifiedName),
}

impl AsRef<CaseInsensitiveString> for Name {
    fn as_ref(&self) -> &CaseInsensitiveString {
        match self {
            Self::Bare(s) => s,
            Self::Typed(t) => t.bare_name(),
        }
    }
}

impl ResolveIntoRef<TypeQualifier> for Name {
    fn resolve_into<T: TypeResolver>(&self, resolver: &T) -> TypeQualifier {
        match self {
            Name::Bare(b) => resolver.resolve(b),
            Name::Typed(t) => t.qualifier(),
        }
    }
}

impl ResolveIntoRef<QualifiedName> for Name {
    fn resolve_into<T: TypeResolver>(&self, resolver: &T) -> QualifiedName {
        match self {
            Name::Bare(s) => QualifiedName::new(s.clone(), resolver.resolve(s)),
            Name::Typed(t) => t.clone(),
        }
    }
}

impl ResolveInto<QualifiedName> for Name {
    fn resolve_into<T: TypeResolver>(self, resolver: &T) -> QualifiedName {
        match self {
            Name::Bare(s) => {
                let qualifier = resolver.resolve(&s);
                QualifiedName::new(s, qualifier)
            }
            Name::Typed(t) => t,
        }
    }
}

impl<S: AsRef<str>> From<S> for Name {
    fn from(s: S) -> Self {
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
