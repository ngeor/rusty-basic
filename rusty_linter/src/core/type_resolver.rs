//! Resolves the type of a name-like expression.
//! For bare names, the type comes from their first character, according to
//! the `DEFINT` etc statements.

use rusty_parser::{BareName, Name, TypeQualifier};

/// Finds the [TypeQualifier] that corresponds to the given character,
/// based on the `DEFINT` etc statements.
pub trait TypeResolver {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier;
}

/// Represents something that can give a [TypeQualifier],
/// with the help of a [TypeResolver].
pub trait IntoTypeQualifier {
    fn qualify(&self, resolver: &impl TypeResolver) -> TypeQualifier;
}

// BareName -> TypeQualifier, based on the first character of the name

impl IntoTypeQualifier for BareName {
    fn qualify(&self, resolver: &impl TypeResolver) -> TypeQualifier {
        let first_char = self.chars().next().unwrap();
        resolver.char_to_qualifier(first_char)
    }
}

// Name -> TypeQualifier. If already qualified, it doesn't use the resolver.

impl IntoTypeQualifier for Name {
    fn qualify(&self, resolver: &impl TypeResolver) -> TypeQualifier {
        match self.qualifier() {
            Some(qualifier) => qualifier,
            _ => self.bare_name().qualify(resolver),
        }
    }
}

/// Converts this object into a qualified object.
pub trait IntoQualified {
    /// The qualified type.
    type Output;

    /// Converts this object into a qualified object.
    fn to_qualified(self, resolver: &impl TypeResolver) -> Self::Output;
}

// BareName -> Name

impl IntoQualified for BareName {
    type Output = Name;

    fn to_qualified(self, resolver: &impl TypeResolver) -> Self::Output {
        let q = self.qualify(resolver);
        Name::qualified(self, q)
    }
}

// Name -> Name. If already qualified, it doesn't use the resolver.

impl IntoQualified for Name {
    type Output = Self;

    fn to_qualified(self, resolver: &impl TypeResolver) -> Self::Output {
        if self.is_bare() {
            let bare_name: BareName = self.into();
            bare_name.to_qualified(resolver)
        } else {
            self
        }
    }
}
