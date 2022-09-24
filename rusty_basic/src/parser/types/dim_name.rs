use std::convert::TryFrom;

use crate::common::*;
use crate::parser::types::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DimName {
    pub bare_name: BareName,
    pub dim_type: DimType,
}

pub type DimNameNode = Locatable<DimName>;
pub type DimNameNodes = Vec<DimNameNode>;

impl DimName {
    pub fn new_compact_local<T>(bare_name: T, qualifier: TypeQualifier) -> Self
    where
        BareName: From<T>,
    {
        Self {
            bare_name: BareName::from(bare_name),
            dim_type: DimType::BuiltIn(qualifier, BuiltInStyle::Compact),
        }
    }

    pub fn new(bare_name: BareName, dim_type: DimType) -> Self {
        Self {
            bare_name,
            dim_type,
        }
    }

    pub fn bare_name(&self) -> &BareName {
        &self.bare_name
    }

    pub fn dim_type(&self) -> &DimType {
        &self.dim_type
    }

    pub fn is_bare(&self) -> bool {
        self.dim_type == DimType::Bare
    }

    pub fn is_built_in_extended(&self) -> Option<TypeQualifier> {
        if let DimType::BuiltIn(q, BuiltInStyle::Extended) = self.dim_type {
            Some(q)
        } else {
            None
        }
    }

    pub fn into_list(self, pos: Location) -> DimList {
        DimList {
            shared: false,
            variables: vec![self.at(pos)],
        }
    }

    #[cfg(test)]
    pub fn into_list_rc(self, row: u32, col: u32) -> DimList {
        self.into_list(Location::new(row, col))
    }

    #[cfg(test)]
    pub fn parse(s: &str) -> Self {
        let qualified_name = QualifiedName::try_from(s).unwrap();
        Self::from(qualified_name)
    }
}

impl From<QualifiedName> for DimName {
    fn from(qualified_name: QualifiedName) -> Self {
        let QualifiedName {
            bare_name,
            qualifier,
        } = qualified_name;
        Self::new_compact_local(bare_name, qualifier)
    }
}

impl DimTypeTrait for DimName {
    fn is_extended(&self) -> bool {
        self.dim_type.is_extended()
    }
}

impl HasExpressionType for DimName {
    fn expression_type(&self) -> ExpressionType {
        self.dim_type.expression_type()
    }
}

impl TryFrom<&DimNameNode> for TypeQualifier {
    type Error = QErrorNode;

    fn try_from(value: &DimNameNode) -> Result<Self, Self::Error> {
        let Locatable { element, .. } = value;
        TypeQualifier::try_from(element).with_err_at(value)
    }
}

impl TryFrom<&DimName> for TypeQualifier {
    type Error = QError;

    fn try_from(value: &DimName) -> Result<Self, Self::Error> {
        TypeQualifier::try_from(value.dim_type())
    }
}

#[derive(Default)]
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

    #[cfg(test)]
    pub fn build_list_rc(self, row: u32, col: u32) -> DimList {
        self.build_list(Location::new(row, col))
    }
}
