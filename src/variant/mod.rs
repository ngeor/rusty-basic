mod array_value;
mod bits;
mod casting;
mod fit;
mod user_defined_type_value;
mod variant;

pub use self::array_value::*;
pub use self::bits::*;
pub use self::user_defined_type_value::*;
pub use self::variant::*;

use crate::common::QError;

pub trait QBNumberCast<T> {
    fn try_cast(&self) -> Result<T, QError>;
}

impl<T> QBNumberCast<Vec<T>> for Vec<Variant>
where
    Variant: QBNumberCast<T>,
{
    fn try_cast(&self) -> Result<Vec<T>, QError> {
        self.iter().map(QBNumberCast::try_cast).collect()
    }
}

/// Calculates the size in bytes of this object.
/// For strings, it is the length in characters, to keep compatibility with
/// the ASCII expectations of QBasic.
pub trait AsciiSize {
    /// Calculates the size in bytes of this object.
    /// For strings, it is the length in characters, to keep compatibility with
    /// the ASCII expectations of QBasic.
    fn ascii_size(&self) -> usize;
}
