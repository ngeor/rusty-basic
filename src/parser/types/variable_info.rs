use crate::parser::{BareName, ExpressionType, TypeQualifier};

/// Additional info for variable expression
#[derive(Clone, Debug, PartialEq)]
pub struct VariableInfo {
    /// The resolved expression type
    pub expression_type: ExpressionType,

    /// Is it a global shared variable
    pub shared: bool,

    pub redim_info: Option<RedimInfo>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RedimInfo {
    pub dimension_count: usize,
}

impl VariableInfo {
    pub fn new_local(expression_type: ExpressionType) -> Self {
        Self {
            expression_type,
            shared: false,
            redim_info: None,
        }
    }

    pub fn new_built_in(q: TypeQualifier, shared: bool) -> Self {
        Self {
            expression_type: ExpressionType::BuiltIn(q),
            shared,
            redim_info: None,
        }
    }

    pub fn new_fixed_length_string(len: u16, shared: bool) -> Self {
        Self {
            expression_type: ExpressionType::FixedLengthString(len),
            shared,
            redim_info: None,
        }
    }

    pub fn new_user_defined(user_defined_type: BareName, shared: bool) -> Self {
        Self {
            expression_type: ExpressionType::UserDefined(user_defined_type),
            shared,
            redim_info: None,
        }
    }

    pub fn unresolved() -> Self {
        Self::new_local(ExpressionType::Unresolved)
    }
}
