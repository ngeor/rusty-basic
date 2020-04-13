use super::{HasQualifier, LetterRangeNode, TypeQualifier};
use crate::common::{HasLocation, Location};

/// Represents a definition of default type, such as DEFINT A-Z.
#[derive(Clone, Debug, PartialEq)]
pub struct DefTypeNode {
    qualifier: TypeQualifier,
    ranges: Vec<LetterRangeNode>,
    pos: Location,
}

impl DefTypeNode {
    pub fn new(qualifier: TypeQualifier, ranges: Vec<LetterRangeNode>, pos: Location) -> Self {
        DefTypeNode {
            qualifier,
            ranges,
            pos,
        }
    }

    pub fn ranges(&self) -> &Vec<LetterRangeNode> {
        &self.ranges
    }
}

impl HasQualifier for DefTypeNode {
    fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }
}

impl HasLocation for DefTypeNode {
    fn location(&self) -> Location {
        self.pos
    }
}
