/// Generic traits

/// Checks if a type can be cast into another type.
pub trait CanCastTo<T> {
    /// Checks if a type can be cast into another type.
    fn can_cast_to(&self, target: T) -> bool;
}
