use crate::common::*;
use crate::parser::types::*;

// same as dim minus the x as string * 5
#[derive(Clone, Debug, PartialEq)]
pub enum Param {
    Bare(BareName),
    Compact(BareName, TypeQualifier),
    ExtendedBuiltIn(BareName, TypeQualifier),
    UserDefined(BareName, BareName),
}

pub type ParamNode = Locatable<Param>;
pub type ParamNodes = Vec<ParamNode>;

impl AsRef<BareName> for Param {
    fn as_ref(&self) -> &BareName {
        match self {
            Self::Bare(n)
            | Self::Compact(n, _)
            | Self::ExtendedBuiltIn(n, _)
            | Self::UserDefined(n, _) => n,
        }
    }
}
