use crate::common::*;
use crate::parser::types::*;
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct DimName {
    pub bare_name: BareName,
    pub dim_type: DimType,
    pub shared: bool,
}

pub type DimNameNode = Locatable<DimName>;
pub type DimNameNodes = Vec<DimNameNode>;

impl DimName {
    pub fn new(bare_name: BareName, dim_type: DimType) -> Self {
        Self {
            bare_name,
            dim_type,
            shared: false,
        }
    }

    pub fn dim_type(&self) -> &DimType {
        &self.dim_type
    }

    pub fn into_inner(self) -> (BareName, DimType) {
        (self.bare_name, self.dim_type)
    }

    pub fn with_shared(self, shared: bool) -> Self {
        let Self {
            bare_name,
            dim_type,
            ..
        } = self;
        Self {
            bare_name,
            dim_type,
            shared,
        }
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

impl DimTypeTrait for DimName {
    fn is_extended(&self) -> bool {
        self.dim_type.is_extended()
    }

    fn is_user_defined(&self) -> Option<&BareNameNode> {
        self.dim_type.is_user_defined()
    }
}

impl DimTypeTrait for DimNameNode {
    fn is_extended(&self) -> bool {
        let dim_name: &DimName = self.as_ref();
        dim_name.is_extended()
    }

    fn is_user_defined(&self) -> Option<&BareNameNode> {
        let dim_name: &DimName = self.as_ref();
        dim_name.is_user_defined()
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

impl<'a> From<&'a DimName> for NameRef<'a> {
    fn from(dim_name: &'a DimName) -> Self {
        let DimName {
            bare_name,
            dim_type,
            ..
        } = dim_name;
        let opt_q: Option<TypeQualifier> = dim_type.into();
        NameRef { bare_name, opt_q }
    }
}

impl<'a> From<&'a DimNameNode> for NameRef<'a> {
    fn from(dim_name_node: &'a DimNameNode) -> Self {
        let dim_name: &DimName = dim_name_node.as_ref();
        dim_name.into()
    }
}
