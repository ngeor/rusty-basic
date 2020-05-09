use super::TypeQualifier;
use crate::common::Locatable;

pub trait HasQualifier {
    fn qualifier(&self) -> TypeQualifier;
}

impl<T: std::fmt::Debug + Sized + HasQualifier> HasQualifier for Locatable<T> {
    fn qualifier(&self) -> TypeQualifier {
        self.as_ref().qualifier()
    }
}
