use rusty_common::*;

/// Calculates the size in bytes of this object.
/// For strings, it is the length in characters, to keep compatibility with
/// the ASCII expectations of QBasic.
pub trait AsciiSize {
    /// Calculates the size in bytes of this object.
    /// For strings, it is the length in characters, to keep compatibility with
    /// the ASCII expectations of QBasic.
    fn ascii_size(&self) -> usize;
}

pub trait QBNumberCast<T> {
    fn try_cast(&self) -> Result<T, QError>;
}
