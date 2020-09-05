use crate::common::Locatable;
use crate::parser::types::{BareName, BareNameNode, ExpressionNode};

#[derive(Clone, Debug, PartialEq)]
pub struct UserDefinedType {
    /// The name of the type
    pub name: BareNameNode,
    /// Comments between the type name and the first element
    pub comments: Vec<Locatable<String>>,
    /// The elements
    pub elements: Vec<ElementNode>,
}

pub type ElementNode = Locatable<Element>;

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    /// The name of the element
    pub name: BareName,
    /// The element type
    pub element_type: ElementType,
    /// Comments between the end of this element and the next one
    pub comments: Vec<Locatable<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Integer,
    Long,
    Single,
    Double,
    String(ExpressionNode),
    UserDefined(BareNameNode),
}
