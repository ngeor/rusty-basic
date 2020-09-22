use crate::common::*;
use crate::parser::types::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DimName {
    Bare(BareName),
    Compact(BareName, TypeQualifier),
    ExtendedBuiltIn(BareName, TypeQualifier),
    String(BareName, ExpressionNode),
    UserDefined(BareName, BareName),
}

pub type DimNameNode = Locatable<DimName>;

impl AsRef<BareName> for DimName {
    fn as_ref(&self) -> &BareName {
        match self {
            Self::Bare(n)
            | Self::Compact(n, _)
            | Self::ExtendedBuiltIn(n, _)
            | Self::String(n, _)
            | Self::UserDefined(n, _) => n,
        }
    }
}
