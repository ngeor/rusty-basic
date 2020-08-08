use super::{NameTrait, TypeQualifier};
use crate::common::*;
use std::convert::TryFrom;

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

#[derive(Clone, Debug, PartialEq)]
pub struct DeclaredName {
    name: CaseInsensitiveString,
    type_definition: TypeDefinition,
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

    pub fn bare_name(&self) -> &CaseInsensitiveString {
        &self.name
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

impl<T: NameTrait> From<T> for DeclaredName {
    fn from(n: T) -> Self {
        match n.opt_qualifier() {
            Some(q) => Self::new(n.into_bare_name(), TypeDefinition::CompactBuiltIn(q)),
            _ => Self::new(n.into_bare_name(), TypeDefinition::Bare),
        }
    }
}

impl<T: NameTrait> From<Locatable<T>> for DeclaredNameNode {
    fn from(name_node: Locatable<T>) -> Self {
        let (name, pos) = name_node.consume();
        let declared_name: DeclaredName = name.into();
        declared_name.at(pos)
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
