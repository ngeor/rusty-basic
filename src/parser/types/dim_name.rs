use crate::common::*;
use crate::parser::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DimName {
    bare_name: BareName,
    dim_type: DimType,
}

pub type DimNameNode = Locatable<DimName>;
pub type DimNameNodes = Vec<DimNameNode>;

impl DimName {
    pub fn new(bare_name: BareName, dim_type: DimType) -> Self {
        Self {
            bare_name,
            dim_type,
        }
    }

    pub fn into_inner(self) -> (BareName, DimType) {
        let Self {
            bare_name,
            dim_type,
        } = self;
        (bare_name, dim_type)
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
