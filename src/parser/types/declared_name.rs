use crate::common::*;
use crate::parser::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DeclaredName {
    pub name: CaseInsensitiveString,
    pub type_definition: TypeDefinition,
}

impl DeclaredName {
    pub fn new(name: CaseInsensitiveString, type_definition: TypeDefinition) -> Self {
        Self {
            name,
            type_definition,
        }
    }

    pub fn bare<S: AsRef<str>>(name: S) -> Self {
        Self::new(name.as_ref().into(), TypeDefinition::Bare)
    }

    pub fn compact<S: AsRef<str>>(name: S, q: TypeQualifier) -> Self {
        Self::new(name.as_ref().into(), TypeDefinition::CompactBuiltIn(q))
    }

    pub fn type_definition(&self) -> &TypeDefinition {
        &self.type_definition
    }
}

pub type DeclaredNameNode = Locatable<DeclaredName>;
pub type DeclaredNameNodes = Vec<DeclaredNameNode>;

// AsRef<BareName>

impl AsRef<BareName> for DeclaredName {
    fn as_ref(&self) -> &BareName {
        &self.name
    }
}
