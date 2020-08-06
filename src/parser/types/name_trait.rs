use super::{QualifiedName, TypeQualifier};
use crate::common::*;

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
