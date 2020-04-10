use super::{QualifiedName, TypeQualifier};
use std::fmt::Display;

pub trait TypeResolver {
    fn resolve(&self, name: &str) -> TypeQualifier;
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Name {
    Bare(String),
    Typed(QualifiedName),
}

#[cfg(test)]
use std::convert::TryFrom;

#[cfg(test)]
impl Name {
    pub fn from<S: AsRef<str>>(s: S) -> Self {
        let mut buf = s.as_ref().to_string();
        let last_ch: char = buf.pop().unwrap();
        match TypeQualifier::try_from(last_ch) {
            Ok(qualifier) => Name::Typed(QualifiedName::new(buf, qualifier)),
            _ => {
                buf.push(last_ch);
                Name::Bare(buf)
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
        assert_eq!(Name::from("A"), Name::Bare("A".to_string()));
        assert_eq!(
            Name::from("Pos%"),
            Name::Typed(QualifiedName::new("Pos", TypeQualifier::PercentInteger))
        );
    }
}
