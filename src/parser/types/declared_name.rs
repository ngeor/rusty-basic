use crate::common::*;
use crate::parser::types::*;
use std::convert::TryFrom;

//
// TypeDefinition
//

#[derive(Clone, Debug, PartialEq)]
pub enum TypeDefinition {
    Bare,
    CompactBuiltIn(TypeQualifier),
    ExtendedBuiltIn(TypeQualifier),
    UserDefined(CaseInsensitiveString),
}

impl TypeDefinition {
    pub fn is_bare(&self) -> bool {
        match self {
            Self::Bare => true,
            _ => false,
        }
    }

    pub fn is_compact_built_in(&self) -> bool {
        match self {
            Self::CompactBuiltIn(_) => true,
            _ => false,
        }
    }

    pub fn is_compact_of_type(&self, q: TypeQualifier) -> bool {
        match self {
            Self::CompactBuiltIn(q_self) => *q_self == q,
            _ => false,
        }
    }

    pub fn is_extended_built_in(&self) -> bool {
        match self {
            Self::ExtendedBuiltIn(_) => true,
            _ => false,
        }
    }

    pub fn is_user_defined(&self) -> bool {
        match self {
            Self::UserDefined(_) => true,
            _ => false,
        }
    }

    pub fn is_built_in(&self) -> bool {
        self.is_compact_built_in() || self.is_extended_built_in()
    }

    pub fn is_extended(&self) -> bool {
        self.is_extended_built_in() || self.is_user_defined()
    }
}

// TypeDefinition -> TypeQualifier

impl TryFrom<&TypeDefinition> for TypeQualifier {
    type Error = bool;
    fn try_from(type_definition: &TypeDefinition) -> Result<Self, bool> {
        match type_definition {
            TypeDefinition::Bare => Err(false),
            TypeDefinition::CompactBuiltIn(q) | TypeDefinition::ExtendedBuiltIn(q) => Ok(*q),
            TypeDefinition::UserDefined(_) => Err(false),
        }
    }
}

//
// DeclaredName
//

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

impl TryFrom<&DeclaredName> for TypeQualifier {
    type Error = bool;
    fn try_from(declared_name: &DeclaredName) -> Result<Self, bool> {
        TypeQualifier::try_from(declared_name.type_definition())
    }
}

impl TryFrom<&DeclaredNameNode> for TypeQualifier {
    type Error = bool;
    fn try_from(declared_name_node: &DeclaredNameNode) -> Result<Self, bool> {
        TypeQualifier::try_from(declared_name_node.as_ref())
    }
}
