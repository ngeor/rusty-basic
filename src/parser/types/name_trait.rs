use super::TypeQualifier;
use crate::common::*;

pub trait NameTrait: Sized + std::fmt::Debug + Clone {
    fn bare_name(&self) -> &CaseInsensitiveString;
    fn opt_qualifier(&self) -> Option<TypeQualifier>;

    /// Checks if the type of this instance is unspecified (bare) or equal to the parameter.
    fn bare_or_eq(&self, other: TypeQualifier) -> bool {
        match self.opt_qualifier() {
            Some(q) => q == other,
            None => true,
        }
    }
}

impl<T: NameTrait> NameTrait for Locatable<T> {
    fn bare_name(&self) -> &CaseInsensitiveString {
        self.as_ref().bare_name()
    }

    fn opt_qualifier(&self) -> Option<TypeQualifier> {
        self.as_ref().opt_qualifier()
    }
}
