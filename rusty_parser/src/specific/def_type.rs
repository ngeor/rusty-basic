use crate::specific::{LetterRange, TypeQualifier};

/// Represents a definition of default type, such as DEFINT A-Z.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefType {
    qualifier: TypeQualifier,
    ranges: Vec<LetterRange>,
}

impl DefType {
    pub fn new(qualifier: TypeQualifier, ranges: Vec<LetterRange>) -> Self {
        Self { qualifier, ranges }
    }

    pub fn ranges(&self) -> &Vec<LetterRange> {
        &self.ranges
    }

    pub fn qualifier(&self) -> TypeQualifier {
        self.qualifier
    }
}
