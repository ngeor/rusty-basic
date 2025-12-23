use std::collections::HashMap;
use std::slice::Iter;

use crate::specific::{
    BareName, BareNamePos, ExpressionPos, ExpressionType, HasExpressionType, Name, TypeQualifier,
};
use rusty_common::Positioned;

#[derive(Clone, Debug, PartialEq)]
pub struct UserDefinedType {
    /// The name of the type
    name: BareName,
    /// Comments between the type name and the first element
    comments: Vec<Positioned<String>>,
    /// The elements
    elements: Vec<ElementPos>,
}

pub type UserDefinedTypes = HashMap<BareName, UserDefinedType>;

impl UserDefinedType {
    pub fn new(
        name: BareName,
        comments: Vec<Positioned<String>>,
        elements: Vec<ElementPos>,
    ) -> Self {
        Self {
            name,
            comments,
            elements,
        }
    }

    pub fn bare_name(&self) -> &BareName {
        &self.name
    }

    pub fn elements(&self) -> Iter<'_, ElementPos> {
        self.elements.iter()
    }
}

pub type ElementPos = Positioned<Element>;

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    /// The name of the element
    pub name: BareName,
    /// The element type
    pub element_type: ElementType,
    /// Comments between the end of this element and the next one
    pub comments: Vec<Positioned<String>>,
}

impl Element {
    pub fn new(
        name: BareName,
        element_type: ElementType,
        comments: Vec<Positioned<String>>,
    ) -> Self {
        Self {
            name,
            element_type,
            comments,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Integer,
    Long,
    Single,
    Double,
    FixedLengthString(ExpressionPos, u16),
    UserDefined(BareNamePos),
}

impl ElementType {
    pub fn can_be_referenced_by_property_name(&self, name: &Name) -> bool {
        match name {
            Name::Bare(_) => true,
            Name::Qualified(_, qualifier) => match self {
                Self::Integer => *qualifier == TypeQualifier::PercentInteger,
                Self::Long => *qualifier == TypeQualifier::AmpersandLong,
                Self::Single => *qualifier == TypeQualifier::BangSingle,
                Self::Double => *qualifier == TypeQualifier::HashDouble,
                Self::FixedLengthString(_, _) => *qualifier == TypeQualifier::DollarString,
                _ => false,
            },
        }
    }
}

impl HasExpressionType for ElementType {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::Integer => ExpressionType::BuiltIn(TypeQualifier::PercentInteger),
            Self::Long => ExpressionType::BuiltIn(TypeQualifier::AmpersandLong),
            Self::Single => ExpressionType::BuiltIn(TypeQualifier::BangSingle),
            Self::Double => ExpressionType::BuiltIn(TypeQualifier::HashDouble),
            Self::FixedLengthString(_, l) => ExpressionType::FixedLengthString(*l),
            Self::UserDefined(Positioned { element, .. }) => {
                ExpressionType::UserDefined(element.clone())
            }
        }
    }
}
