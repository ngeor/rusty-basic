use crate::parser::{BareName, BareNameNode, ExpressionType, HasExpressionType};

/// A variable name with a type.
///
/// This is an abstraction to address the similarities between [DimName]
/// and [ParamName].
#[derive(Clone, Debug, PartialEq)]
pub struct VarName<T> {
    // TODO make fields private
    pub bare_name: BareName,
    pub var_type: T,
}

impl<T> VarName<T> {
    pub fn new(bare_name: BareName, var_type: T) -> Self {
        Self {
            bare_name,
            var_type,
        }
    }

    pub fn bare_name(&self) -> &BareName {
        &self.bare_name
    }

    pub fn var_type(&self) -> &T {
        &self.var_type
    }
}

impl<T> HasExpressionType for VarName<T>
where
    T: HasExpressionType,
{
    fn expression_type(&self) -> ExpressionType {
        self.var_type.expression_type()
    }
}

pub trait VarTypeToArray {
    type ArrayType;

    /// Converts the variable type to an array variable type,
    /// as long as the `array_type` is not empty.
    fn to_array(self, array_type: Self::ArrayType) -> Self;
}

pub trait VarTypeUserDefined {
    fn from_user_defined(name_node: BareNameNode) -> Self;
}
