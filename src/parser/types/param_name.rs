use crate::common::*;
use crate::parser::types::*;

// same as dim minus the x as string * 5
#[derive(Clone, Debug, PartialEq)]
pub struct ParamName {
    bare_name: BareName,
    param_type: ParamType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParamType {
    Bare,
    BuiltIn(TypeQualifier, BuiltInStyle),
    UserDefined(BareNameNode),
    Array(Box<ParamType>),
}

pub type ParamNameNode = Locatable<ParamName>;
pub type ParamNameNodes = Vec<ParamNameNode>;

impl ParamName {
    pub fn new(bare_name: BareName, param_type: ParamType) -> Self {
        Self {
            bare_name,
            param_type,
        }
    }

    pub fn param_type(&self) -> &ParamType {
        &self.param_type
    }

    pub fn into_inner(self) -> (BareName, ParamType) {
        let Self {
            bare_name,
            param_type,
        } = self;
        (bare_name, param_type)
    }
}

impl AsRef<BareName> for ParamName {
    fn as_ref(&self) -> &BareName {
        &self.bare_name
    }
}
