use crate::common::*;
use crate::parser::types::*;

// same as dim minus the x as string * 5
#[derive(Clone, Debug, PartialEq)]
pub enum ParamName {
    Bare(BareName),
    Compact(BareName, TypeQualifier),
    ExtendedBuiltIn(BareName, TypeQualifier),
    UserDefined(BareName, BareName),
}

pub type ParamNameNode = Locatable<ParamName>;
pub type ParamNameNodes = Vec<ParamNameNode>;

impl AsRef<BareName> for ParamName {
    fn as_ref(&self) -> &BareName {
        match self {
            Self::Bare(n)
            | Self::Compact(n, _)
            | Self::ExtendedBuiltIn(n, _)
            | Self::UserDefined(n, _) => n,
        }
    }
}
