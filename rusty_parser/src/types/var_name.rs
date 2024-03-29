use crate::{BareName, BareNamePos, ExpressionType, HasExpressionType, TypeQualifier};

/// A variable name with a type.
///
/// This is an abstraction to address the similarities between [DimName]
/// and [ParamName].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedName<T> {
    // TODO make fields private
    pub bare_name: BareName,
    pub var_type: T,
}

impl<T> TypedName<T> {
    pub fn new(bare_name: BareName, var_type: T) -> Self {
        Self {
            bare_name,
            var_type,
        }
    }
}

impl<T> HasExpressionType for TypedName<T>
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

pub trait VarTypeNewUserDefined {
    fn new_user_defined(bare_name_pos: BareNamePos) -> Self;
}

pub trait VarTypeToUserDefinedRecursively {
    fn as_user_defined_recursively(&self) -> Option<&BareNamePos>;
}

pub trait VarTypeQualifier {
    fn to_qualifier_recursively(&self) -> Option<TypeQualifier>;
}

pub trait VarTypeIsExtended {
    fn is_extended(&self) -> bool;
}

pub trait VarTypeNewBuiltInCompact {
    fn new_built_in_compact(q: TypeQualifier) -> Self;
}

pub trait VarTypeNewBuiltInExtended {
    fn new_built_in_extended(q: TypeQualifier) -> Self;
}
