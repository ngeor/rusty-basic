use rusty_variant::{UserDefinedTypeValue, VArray, Variant};

/// Calculates the size in bytes of this object.
/// For strings, it is the length in characters, to keep compatibility with
/// the ASCII expectations of QBasic.
pub trait QByteSize {
    /// Calculates the size in bytes of this object.
    /// For strings, it is the length in characters, to keep compatibility with
    /// the ASCII expectations of QBasic.
    fn byte_size(&self) -> usize;
}

impl QByteSize for Variant {
    fn byte_size(&self) -> usize {
        match self {
            Self::VInteger(_) => 2,
            Self::VLong(_) | Self::VSingle(_) => 4,
            Self::VDouble(_) => 8,
            Self::VString(s) => s.chars().count(),
            Self::VArray(v_array) => v_array.byte_size(),
            Self::VUserDefined(user_defined_type_value) => user_defined_type_value.byte_size(),
        }
    }
}

impl QByteSize for VArray {
    fn byte_size(&self) -> usize {
        self.len() * self.first().byte_size()
    }
}

impl QByteSize for UserDefinedTypeValue {
    fn byte_size(&self) -> usize {
        self.values().map(Variant::byte_size).sum()
    }
}
