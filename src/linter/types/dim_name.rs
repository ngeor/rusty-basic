use crate::common::{CanCastTo, Locatable};
use crate::linter::{ElementType, HasTypeDefinition, TypeDefinition};
use crate::parser::{BareName, QualifiedName, TypeQualifier};
#[cfg(test)]
use std::convert::TryFrom;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DimName {
    // A -> A!
    // A AS STRING
    // A$, A% etc
    BuiltIn(BareName, TypeQualifier),

    // DIM C AS Card
    UserDefined(UserDefinedName),

    /// DIM X AS STRING * 1
    String(BareName, u16),

    // C.Suit, Name.Address, Name.Address.PostCode
    Many(UserDefinedName, Members),
}

pub type DimNameNode = Locatable<DimName>;
pub type DimNameNodes = Vec<DimNameNode>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UserDefinedName {
    pub name: BareName,
    pub type_name: BareName,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    pub fn built_in(qualified_name: QualifiedName) -> Self {
        let QualifiedName { name, qualifier } = qualified_name;
        Self::BuiltIn(name, qualifier)
    }

    #[cfg(test)]
    pub fn parse(s: &str) -> Self {
        Self::built_in(QualifiedName::try_from(s).unwrap())
    }

    #[cfg(test)]
    pub fn user_defined(name: &str, type_name: &str) -> Self {
        Self::UserDefined(UserDefinedName {
            name: name.into(),
            type_name: type_name.into(),
        })
    }

    pub fn append(self, members: Members) -> Self {
        match self {
            Self::BuiltIn(_, _) | Self::String(_, _) => {
                panic!("Cannot append members to built-in resolved name")
            }
            Self::UserDefined(user_defined_name) => Self::Many(user_defined_name, members),
            Self::Many(user_defined_name, existing_members) => {
                Self::Many(user_defined_name, existing_members.append(members))
            }
        }
    }
}

impl AsRef<BareName> for DimName {
    fn as_ref(&self) -> &BareName {
        match self {
            Self::BuiltIn(name, _) | Self::String(name, _) => name,
            Self::UserDefined(UserDefinedName { name, .. }) => name,
            Self::Many(UserDefinedName { name, .. }, _) => name,
        }
    }
}

impl HasTypeDefinition for DimName {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::BuiltIn(_, qualifier) => TypeDefinition::BuiltIn(*qualifier),
            Self::String(_, len) => TypeDefinition::String(*len),
            Self::UserDefined(UserDefinedName { type_name, .. }) => {
                TypeDefinition::UserDefined(type_name.clone())
            }
            Self::Many(_, members) => members.type_definition(),
        }
    }
}

impl CanCastTo<TypeQualifier> for DimName {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.type_definition().can_cast_to(other)
    }
}
