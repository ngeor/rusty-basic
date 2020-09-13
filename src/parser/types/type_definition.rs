use crate::common::CaseInsensitiveString;
use crate::parser::types::TypeQualifier;

#[derive(Clone, Debug, PartialEq)]
pub enum TypeDefinition {
    Bare,
    CompactBuiltIn(TypeQualifier),
    ExtendedBuiltIn(TypeQualifier),
    // TODO add test proving it is not possible to have a parameter like X AS STRING * 8
    // TODO add test passing a STRING(len) into a STRING parameter and show it's getting truncated AFTER the sub exits
    UserDefined(CaseInsensitiveString),
}

impl TypeDefinition {
    pub fn is_bare(&self) -> bool {
        match self {
            Self::Bare => true,
            _ => false,
        }
    }

    pub fn is_compact_built_in(&self) -> bool {
        match self {
            Self::CompactBuiltIn(_) => true,
            _ => false,
        }
    }

    pub fn is_compact_of_type(&self, q: TypeQualifier) -> bool {
        match self {
            Self::CompactBuiltIn(q_self) => *q_self == q,
            _ => false,
        }
    }

    pub fn is_extended_built_in(&self) -> bool {
        match self {
            Self::ExtendedBuiltIn(_) => true,
            _ => false,
        }
    }

    pub fn is_user_defined(&self) -> bool {
        match self {
            Self::UserDefined(_) => true,
            _ => false,
        }
    }

    pub fn is_built_in(&self) -> bool {
        self.is_compact_built_in() || self.is_extended_built_in()
    }

    pub fn is_extended(&self) -> bool {
        self.is_extended_built_in() || self.is_user_defined()
    }
}
