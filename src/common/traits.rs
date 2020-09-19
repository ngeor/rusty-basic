/// Generic traits
use super::Locatable;

/// Checks if a type can be cast into another type.
pub trait CanCastTo<T> {
    /// Checks if a type can be cast into another type.
    fn can_cast_to(&self, other: T) -> bool;
}

impl<T, U> CanCastTo<U> for Locatable<T>
where
    T: CanCastTo<U>,
{
    fn can_cast_to(&self, other: U) -> bool {
        self.as_ref().can_cast_to(other)
    }
}
