use crate::types::*;
use rusty_common::*;
#[cfg(test)]
use std::convert::TryFrom;

pub type DimName = VarName<DimType>;
pub type DimNameNode = Locatable<DimName>;
pub type DimNameNodes = Vec<DimNameNode>;

impl DimName {
    pub fn new_compact_local<T>(bare_name: T, qualifier: TypeQualifier) -> Self
    where
        BareName: From<T>,
    {
        Self {
            bare_name: BareName::from(bare_name),
            var_type: DimType::BuiltIn(qualifier, BuiltInStyle::Compact),
        }
    }

    pub fn into_list(self, pos: Location) -> DimList {
        DimList {
            shared: false,
            variables: vec![self.at(pos)],
        }
    }

    #[cfg(any(test, feature = "test_utils"))]
    pub fn into_list_rc(self, row: u32, col: u32) -> DimList {
        self.into_list(Location::new(row, col))
    }

    #[cfg(any(test, feature = "test_utils"))]
    pub fn parse(s: &str) -> Self {
        let qualified_name = QualifiedName::try_from(s).unwrap();
        Self::from(qualified_name)
    }
}

impl From<QualifiedName> for DimName {
    fn from(qualified_name: QualifiedName) -> Self {
        let (bare_name, qualifier) = qualified_name.into_inner();
        Self::new_compact_local(bare_name, qualifier)
    }
}

#[derive(Default)]
// TODO #[deprecated]
pub struct DimNameBuilder {
    pub bare_name: Option<BareName>,
    pub dim_type: Option<DimType>,
}

impl DimNameBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bare_name<T>(mut self, bare_name: T) -> Self
    where
        BareName: From<T>,
    {
        self.bare_name = Some(BareName::from(bare_name));
        self
    }

    pub fn dim_type(mut self, dim_type: DimType) -> Self {
        self.dim_type = Some(dim_type);
        self
    }

    pub fn build(self) -> DimName {
        DimName::new(self.bare_name.unwrap(), self.dim_type.unwrap())
    }

    pub fn build_list(self, pos: Location) -> DimList {
        DimList {
            shared: false,
            variables: vec![self.build().at(pos)],
        }
    }

    #[cfg(any(test, feature = "test_utils"))]
    pub fn build_list_rc(self, row: u32, col: u32) -> DimList {
        self.build_list(Location::new(row, col))
    }
}
