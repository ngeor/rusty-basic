use crate::common::*;
use crate::parser::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DimName {
    bare_name: BareName,
    dim_type: DimType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DimType {
    Bare,
    Compact(TypeQualifier),
    Extended(TypeQualifier),
    FixedLengthString(ExpressionNode),
    UserDefined(BareNameNode),
}

pub type DimNameNode = Locatable<DimName>;

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
}

impl AsRef<BareName> for DimName {
    fn as_ref(&self) -> &BareName {
        &self.bare_name
    }
}
