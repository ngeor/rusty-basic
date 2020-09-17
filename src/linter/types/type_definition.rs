use crate::parser;
use crate::parser::{BareName, TypeQualifier};

/// A linted (resolved) `TypeDefinition`.
///
/// Similar to the one defined in `parser` but without `Bare` and with `FileHandle`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeDefinition {
    BuiltIn(TypeQualifier),
    String(u32),
    UserDefined(BareName),
    FileHandle,
}

impl From<parser::TypeDefinition> for TypeDefinition {
    // TODO is this used
    fn from(type_definition: parser::TypeDefinition) -> Self {
        match type_definition {
            parser::TypeDefinition::Bare => panic!("Unresolved bare type"), // as this is internal error, it is ok to panic
            parser::TypeDefinition::CompactBuiltIn(q)
            | parser::TypeDefinition::ExtendedBuiltIn(q) => Self::BuiltIn(q),
            parser::TypeDefinition::UserDefined(bare_name) => Self::UserDefined(bare_name),
        }
    }
}
