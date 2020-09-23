use crate::common::{CanCastTo, Locatable};
use crate::linter::{ElementType, HasTypeDefinition, TypeDefinition};
use crate::parser::{BareName, QualifiedName, TypeQualifier};
#[cfg(test)]
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct DimName {
    bare_name: BareName,
    dim_type: DimType,
}

pub type DimNameNode = Locatable<DimName>;
pub type DimNameNodes = Vec<DimNameNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    // A -> A!
    // A AS STRING
    // A$, A% etc
    BuiltIn(TypeQualifier),

    // DIM C AS Card
    UserDefined(BareName),

    /// DIM X AS STRING * 1
    FixedLengthString(u16),

    // C.Suit, Name.Address, Name.Address.PostCode
    Many(BareName, Members),
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserDefinedName {
    pub name: BareName,
    pub type_name: BareName,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Members {
    Leaf {
        name: BareName,
        element_type: ElementType,
    },
    Node(UserDefinedName, Box<Members>),
}

impl Members {
    pub fn name_path(&self) -> Vec<BareName> {
        match self {
            Self::Leaf { name, .. } => vec![name.clone()],
            Self::Node(UserDefinedName { name, .. }, boxed_members) => {
                let mut result = vec![name.clone()];
                result.extend(boxed_members.name_path());
                result
            }
        }
    }

    pub fn append(self, other: Self) -> Self {
        match self {
            Self::Leaf { name, element_type } => match element_type {
                ElementType::UserDefined(type_name) => {
                    Self::Node(UserDefinedName { name, type_name }, Box::new(other))
                }
                _ => panic!("Cannot extend leaf element which is not user defined type"),
            },
            Self::Node(user_defined_name, boxed_members) => {
                Self::Node(user_defined_name, Box::new(boxed_members.append(other)))
            }
        }
    }
}

impl HasTypeDefinition for Members {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::Leaf { element_type, .. } => element_type.type_definition(),
            Self::Node(_, boxed_members) => boxed_members.type_definition(),
        }
    }
}

impl DimName {
    pub fn new(bare_name: BareName, dim_type: DimType) -> Self {
        Self {
            bare_name,
            dim_type,
        }
    }

    pub fn dim_type(&self) -> &DimType {
        &self.dim_type
    }

    pub fn into_inner(self) -> (BareName, DimType) {
        (self.bare_name, self.dim_type)
    }

    pub fn built_in(qualified_name: QualifiedName) -> Self {
        let QualifiedName {
            bare_name: name,
            qualifier,
        } = qualified_name;
        Self::new(name, DimType::BuiltIn(qualifier))
    }

    #[cfg(test)]
    pub fn parse(s: &str) -> Self {
        Self::built_in(QualifiedName::try_from(s).unwrap())
    }

    #[cfg(test)]
    pub fn user_defined(name: &str, type_name: &str) -> Self {
        Self::new(name.into(), DimType::UserDefined(type_name.into()))
    }

    pub fn append(self, members: Members) -> Self {
        let Self {
            bare_name,
            dim_type,
        } = self;
        match dim_type {
            DimType::BuiltIn(_) | DimType::FixedLengthString(_) => {
                panic!("Cannot append members to built-in resolved name")
            }
            DimType::UserDefined(user_defined_type) => {
                Self::new(bare_name, DimType::Many(user_defined_type, members))
            }
            DimType::Many(user_defined_name, existing_members) => Self::new(
                bare_name,
                DimType::Many(user_defined_name, existing_members.append(members)),
            ),
        }
    }
}

impl AsRef<BareName> for DimName {
    fn as_ref(&self) -> &BareName {
        &self.bare_name
    }
}
impl HasTypeDefinition for DimName {
    fn type_definition(&self) -> TypeDefinition {
        self.dim_type().type_definition()
    }
}

impl HasTypeDefinition for DimType {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::BuiltIn(qualifier) => TypeDefinition::BuiltIn(*qualifier),
            Self::FixedLengthString(len) => TypeDefinition::String(*len),
            Self::UserDefined(type_name) => TypeDefinition::UserDefined(type_name.clone()),
            Self::Many(_, members) => members.type_definition(),
        }
    }
}

impl CanCastTo<TypeQualifier> for DimName {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.type_definition().can_cast_to(other)
    }
}
