use crate::common::*;
use crate::parser::types::*;
use std::convert::TryFrom;

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

    pub fn new<T>(bare_name: T, dim_type: DimType) -> Self
    where
        BareName: From<T>,
    {
        Self {
            bare_name: BareName::from(bare_name),
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

    pub fn with_dim_type(self, dim_type: DimType) -> Self {
        let Self { bare_name, .. } = self;
        Self {
            bare_name,
            dim_type,
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
        Self::new_compact_local(bare_name, qualifier)
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
}

#[cfg(test)]
impl DimNameNode {
    #[cfg(test)]
    pub fn into_list(self) -> DimList {
        DimList {
            shared: false,
            variables: vec![self],
        }
    }
}
