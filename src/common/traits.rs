/// Generic traits

/// Checks if a type can be cast into another type.
pub trait CanCastTo<T> {
    /// Checks if a type can be cast into another type.
    fn can_cast_to(&self, other: T) -> bool;
}

/// Tries to convert the current type into type `T`.
pub trait TryRefInto<T> {
    /// The error returned if the conversion fails.
    type Error;

    /// Tries to convert the current type into type `T`.
    fn try_ref_into(&self) -> Result<T, Self::Error>;
}

/// Tries to get a reference from the current type into type `T`.
pub trait TryAsRef<T> {
    /// The error returned if the conversion fails.
    type Error;

    /// Tries to get a reference from the current type into type `T`.
    fn try_as_ref(&self) -> Result<&T, Self::Error>;
}
