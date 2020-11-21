use crate::common::*;
use crate::parser::types::*;

#[cfg(test)]
use std::convert::TryFrom;
use crate::parser::DimType::BuiltIn;

#[derive(Clone, Debug, PartialEq)]
pub struct DimName {
    pub bare_name: BareName,
    pub dim_type: DimType,
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

    pub fn dim_type(&self) -> &DimType {
        &self.dim_type
    }

    pub fn into_inner(self) -> (BareName, DimType) {
        (self.bare_name, self.dim_type)
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
        Self::new(
            bare_name,
            DimType::BuiltIn(qualifier, BuiltInStyle::Compact),
        )
    }
}

impl AsRef<BareName> for DimName {
    fn as_ref(&self) -> &BareName {
        &self.bare_name
    }
}

impl AsRef<BareName> for DimNameNode {
    fn as_ref(&self) -> &BareName {
        let dim_name: &DimName = self.as_ref();
        dim_name.as_ref()
    }
}

pub trait DimNameTrait {
    fn is_bare(&self) -> bool;

    fn is_built_in_extended(&self) -> Option<TypeQualifier>;
}

impl DimNameTrait for DimName {
    fn is_bare(&self) -> bool {
        self.dim_type == DimType::Bare
    }

    fn is_built_in_extended(&self) -> Option<TypeQualifier> {
        if let DimType::BuiltIn(q, BuiltInStyle::Extended) = self.dim_type {
            Some(q)
        } else {
            None
        }
    }
}

impl DimNameTrait for DimNameNode {
    fn is_bare(&self) -> bool {
        let dim_name: &DimName = self.as_ref();
        dim_name.is_bare()
    }

    fn is_built_in_extended(&self) -> Option<TypeQualifier> {
        let dim_name: &DimName = self.as_ref();
        dim_name.is_built_in_extended()
    }
}
