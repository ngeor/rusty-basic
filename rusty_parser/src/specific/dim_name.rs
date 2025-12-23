use crate::specific::*;
use rusty_common::*;
#[cfg(test)]
use std::convert::TryFrom;

pub type DimVar = TypedName<DimType>;
pub type DimVarPos = Positioned<DimVar>;
pub type DimVars = Vec<DimVarPos>;

impl DimVar {
    pub fn new_compact_local<T>(bare_name: T, qualifier: TypeQualifier) -> Self
    where
        BareName: From<T>,
    {
        Self::new(
            BareName::from(bare_name),
            DimType::BuiltIn(qualifier, BuiltInStyle::Compact),
        )
    }

    pub fn into_list(self, pos: Position) -> DimList {
        DimList {
            shared: false,
            variables: vec![self.at_pos(pos)],
        }
    }

    // TODO #[cfg(test)]
    pub fn into_list_rc(self, row: u32, col: u32) -> DimList {
        self.into_list(Position::new(row, col))
    }

    // TODO #[cfg(test)]
    pub fn parse(s: &str) -> Self {
        let qualified_name = QualifiedName::try_from(s).unwrap();
        Self::from(qualified_name)
    }
}

impl From<QualifiedName> for DimVar {
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

    pub fn build(self) -> DimVar {
        DimVar::new(self.bare_name.unwrap(), self.dim_type.unwrap())
    }

    pub fn build_list(self, pos: Position) -> DimList {
        DimList {
            shared: false,
            variables: vec![self.build().at_pos(pos)],
        }
    }

    // TODO #[cfg(test)]
    pub fn build_list_rc(self, row: u32, col: u32) -> DimList {
        self.build_list(Position::new(row, col))
    }
}
