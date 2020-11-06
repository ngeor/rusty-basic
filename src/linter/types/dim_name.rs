use crate::common::{CanCastTo, Locatable, QError, StringUtils};
use crate::linter::{ExpressionNode, ExpressionType, HasExpressionType, UserDefinedTypes};
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
use crate::variant::{UserDefinedTypeValue, Variant};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct DimName {
    pub bare_name: BareName,
    pub dim_type: DimType,
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

    Array(ArrayDimensions, Box<ExpressionType>),
}

pub type ArrayDimensions = Vec<ArrayDimension>;

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayDimension {
    pub lbound: ExpressionNode,
    pub ubound: ExpressionNode,
}

impl DimType {
    pub fn default_variant(&self, types: &UserDefinedTypes) -> Variant {
        match self {
            Self::BuiltIn(q) => Variant::from(*q),
            Self::FixedLengthString(len) => String::new().fix_length(*len as usize).into(),
            Self::UserDefined(type_name) => {
                Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(type_name, types)))
            }
            Self::Array(_, _) => panic!("not possible to evaluate a default value for an array"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserDefinedName {
    pub name: BareName,
    pub type_name: BareName,
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

impl TryFrom<DimName> for QualifiedName {
    type Error = QError;

    fn try_from(value: DimName) -> Result<Self, Self::Error> {
        let (bare_name, dim_type) = value.into_inner();
        match dim_type {
            DimType::BuiltIn(q) => Ok(QualifiedName::new(bare_name, q)),
            DimType::FixedLengthString(_) => {
                Ok(QualifiedName::new(bare_name, TypeQualifier::DollarString))
            }
            _ => Err(QError::TypeMismatch),
        }
    }
}

impl From<DimName> for Name {
    fn from(dim_name: DimName) -> Self {
        let (bare_name, dim_type) = dim_name.into_inner();
        match dim_type {
            DimType::BuiltIn(qualifier) => Name::new(bare_name, Some(qualifier)),
            DimType::FixedLengthString(_len) => {
                Name::new(bare_name, Some(TypeQualifier::DollarString))
            }
            DimType::UserDefined(_) => Name::new(bare_name, None),
            DimType::Array(_, box_element_type) => {
                let element_type = *box_element_type;
                match element_type {
                    ExpressionType::BuiltIn(q) => Name::new(bare_name, Some(q)),
                    ExpressionType::FixedLengthString(_) => {
                        Name::new(bare_name, Some(TypeQualifier::DollarString))
                    }
                    _ => Name::new(bare_name, None),
                }
            }
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

    #[cfg(test)]
    pub fn parse(s: &str) -> Self {
        let qualified_name = QualifiedName::try_from(s).unwrap();
        qualified_name.into()
    }

    #[cfg(test)]
    pub fn user_defined(name: &str, type_name: &str) -> Self {
        Self::new(name.into(), DimType::UserDefined(type_name.into()))
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
            Self::Array(_, element_type) => element_type.as_ref().clone(),
        }
    }
}

impl CanCastTo<TypeQualifier> for DimName {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.expression_type().can_cast_to(other)
    }
}
