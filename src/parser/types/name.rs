use super::{HasQualifier, TypeQualifier};
use crate::common::{CaseInsensitiveString, Locatable};
use std::convert::TryFrom;

//
// NameTrait
//

pub trait NameTrait: Sized + std::fmt::Debug + Clone {
    fn bare_name(&self) -> &CaseInsensitiveString;
    fn opt_qualifier(&self) -> Option<TypeQualifier>;
    fn consume_bare_name(self) -> CaseInsensitiveString;

    /// Checks if the type of this instance is unspecified (bare) or equal to the parameter.
    fn bare_or_eq(&self, other: TypeQualifier) -> bool {
        match self.opt_qualifier() {
            Some(q) => q == other,
            None => true,
        }
    }

    fn with_type(self, q: TypeQualifier) -> QualifiedName {
        QualifiedName::new(self.consume_bare_name(), q)
    }

    fn with_type_ref(&self, q: TypeQualifier) -> QualifiedName {
        QualifiedName::new(self.bare_name().clone(), q)
    }
}

impl<T: NameTrait> NameTrait for Locatable<T> {
    fn bare_name(&self) -> &CaseInsensitiveString {
        self.as_ref().bare_name()
    }

    fn consume_bare_name(self) -> CaseInsensitiveString {
        self.consume().0.consume_bare_name()
    }

    fn opt_qualifier(&self) -> Option<TypeQualifier> {
        self.as_ref().opt_qualifier()
    }
}

//
// BareName
//

pub type BareName = CaseInsensitiveString;
pub type BareNameNode = Locatable<BareName>;

impl NameTrait for BareName {
    fn bare_name(&self) -> &CaseInsensitiveString {
        self
    }

    fn consume_bare_name(self) -> CaseInsensitiveString {
        self
    }

    fn opt_qualifier(&self) -> Option<TypeQualifier> {
        None
    }
}

//
// QualifiedName
//


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QualifiedName {
    name: CaseInsensitiveString,
    qualifier: TypeQualifier,
}

impl QualifiedName {
    pub fn new(name: CaseInsensitiveString, qualifier: TypeQualifier) -> Self {
        QualifiedName { name, qualifier }
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

    fn consume_bare_name(self) -> CaseInsensitiveString {
        self.name
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


//
// Name
//

#[derive(Clone, Debug, PartialEq)]
pub enum Name {
    Bare(CaseInsensitiveString),
    Qualified(QualifiedName),
}
pub type NameNode = Locatable<Name>;

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
}

impl NameTrait for Name {
    fn bare_name(&self) -> &CaseInsensitiveString {
        match self {
            Self::Bare(b) => b,
            Self::Qualified(t) => t.bare_name(),
        }
    }

    fn consume_bare_name(self) -> CaseInsensitiveString {
        match self {
            Self::Bare(b) => b,
            Self::Qualified(q) => q.consume_bare_name(),
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
