use crate::common::Locatable;
use crate::parser::types::{BareName, BareNameNode, ExpressionNode};
use std::slice::Iter;

#[derive(Clone, Debug, PartialEq)]
pub struct UserDefinedType {
    /// The name of the type
    name_node: BareNameNode,
    /// Comments between the type name and the first element
    comments: Vec<Locatable<String>>,
    /// The elements
    elements: Vec<ElementNode>,
}

impl UserDefinedType {
    pub fn new(
        name_node: BareNameNode,
        comments: Vec<Locatable<String>>,
        elements: Vec<ElementNode>,
    ) -> Self {
        Self {
            name_node,
            comments,
            elements,
        }
    }

    pub fn elements(&self) -> Iter<'_, ElementNode> {
        self.elements.iter()
    }
}

impl AsRef<BareName> for UserDefinedType {
    fn as_ref(&self) -> &BareName {
        self.name_node.as_ref()
    }
}

pub type ElementNode = Locatable<Element>;

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    /// The name of the element
    name: BareName,
    /// The element type
    element_type: ElementType,
    /// Comments between the end of this element and the next one
    comments: Vec<Locatable<String>>,
}

impl Element {
    pub fn new(
        name: BareName,
        element_type: ElementType,
        comments: Vec<Locatable<String>>,
    ) -> Self {
        Self {
            name,
            element_type,
            comments,
        }
    }

    pub fn element_type(&self) -> &ElementType {
        &self.element_type
    }
}

impl AsRef<BareName> for Element {
    fn as_ref(&self) -> &BareName {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Integer,
    Long,
    Single,
    Double,
    FixedLengthString(ExpressionNode, u16),
    UserDefined(BareNameNode),
}
