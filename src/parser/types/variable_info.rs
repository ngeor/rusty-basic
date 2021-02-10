use crate::parser::ExpressionType;

/// Additional info for variable expression
#[derive(Clone, Debug, PartialEq)]
pub struct VariableInfo {
    /// The resolved expression type
    pub expression_type: ExpressionType,

    /// Is it a global shared variable
    pub shared: bool,
}

impl VariableInfo {
    pub fn new_local(expression_type: ExpressionType) -> Self {
        Self {
            expression_type,
            shared: false,
        }
    }

    pub fn unresolved() -> Self {
        Self::new_local(ExpressionType::Unresolved)
    }
}
