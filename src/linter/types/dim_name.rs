use crate::common::{CanCastTo, Locatable, StringUtils};
use crate::linter::{ElementType, ExpressionType, HasExpressionType, UserDefinedTypes};
use crate::parser::{BareName, QualifiedName, TypeQualifier};
use crate::variant::{UserDefinedTypeValue, Variant};
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

impl DimType {
    pub fn default_variant(&self, types: &UserDefinedTypes) -> Variant {
        match self {
            Self::BuiltIn(q) => Variant::from(*q),
            Self::FixedLengthString(len) => String::new().fix_length(*len as usize).into(),
            Self::UserDefined(type_name) => {
                Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(type_name, types)))
            }
            Self::Many(_, _) => panic!("not possible to declare a variable of type Many"),
        }
    }
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

impl HasExpressionType for Members {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::Leaf { element_type, .. } => element_type.expression_type(),
            Self::Node(_, boxed_members) => boxed_members.expression_type(),
        }
    }
}

impl From<QualifiedName> for DimName {
    fn from(qualified_name: QualifiedName) -> Self {
        let QualifiedName {
            bare_name,
            qualifier,
        } = qualified_name;
        Self::new(bare_name, DimType::BuiltIn(qualifier))
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

    #[cfg(test)]
    pub fn parse(s: &str) -> Self {
        let qualified_name = QualifiedName::try_from(s).unwrap();
        qualified_name.into()
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
impl HasExpressionType for DimName {
    fn expression_type(&self) -> ExpressionType {
        self.dim_type().expression_type()
    }
}

impl HasExpressionType for DimType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::BuiltIn(qualifier) => ExpressionType::BuiltIn(*qualifier),
            Self::FixedLengthString(len) => ExpressionType::FixedLengthString(*len),
            Self::UserDefined(type_name) => ExpressionType::UserDefined(type_name.clone()),
            Self::Many(_, members) => members.expression_type(),
        }
    }
}

impl CanCastTo<TypeQualifier> for DimName {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.expression_type().can_cast_to(other)
    }
}