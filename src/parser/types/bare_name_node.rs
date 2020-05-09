use super::{NameTrait, TypeQualifier};
use crate::common::{CaseInsensitiveString, Locatable};

pub type BareName = CaseInsensitiveString;
pub type BareNameNode = Locatable<BareName>;

impl NameTrait for BareName {
    fn bare_name(&self) -> &CaseInsensitiveString {
        self
    }

    fn opt_qualifier(&self) -> Option<TypeQualifier> {
        None
    }
}
