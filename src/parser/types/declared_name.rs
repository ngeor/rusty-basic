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

    pub fn is_bare(&self) -> bool {
        self.type_definition.is_bare()
    }

    pub fn is_compact_built_in(&self) -> bool {
        self.type_definition.is_compact_built_in()
    }

    pub fn is_compact_of_type(&self, q: TypeQualifier) -> bool {
        self.type_definition.is_compact_of_type(q)
    }

    pub fn is_extended_built_in(&self) -> bool {
        self.type_definition.is_extended_built_in()
    }

    pub fn is_user_defined(&self) -> bool {
        self.type_definition.is_user_defined()
    }

    pub fn is_built_in(&self) -> bool {
        self.type_definition.is_built_in()
    }

    pub fn is_extended(&self) -> bool {
        self.type_definition.is_extended()
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

impl From<DeclaredName> for (BareName, TypeDefinition) {
    fn from(n: DeclaredName) -> (BareName, TypeDefinition) {
        (n.name, n.type_definition)
    }
}

// BareName -> DeclaredName

impl From<BareName> for DeclaredName {
    fn from(n: BareName) -> Self {
        Self::new(n, TypeDefinition::Bare)
    }
}

// QualifiedName -> DeclaredName

impl From<QualifiedName> for DeclaredName {
    fn from(q_name: QualifiedName) -> Self {
        let QualifiedName { name, qualifier } = q_name;
        Self::new(name, TypeDefinition::CompactBuiltIn(qualifier))
    }
}

// Name -> DeclaredName

impl From<Name> for DeclaredName {
    fn from(n: Name) -> Self {
        match n {
            Name::Bare(b) => b.into(),
            Name::Qualified { name, qualifier } => {
                DeclaredName::new(name, TypeDefinition::CompactBuiltIn(qualifier))
            }
        }
    }
}
