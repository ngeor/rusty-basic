use crate::{FunctionMap, ResolvedParamType, SubMap};
use rusty_parser::specific::{
    Expression, ExpressionPos, ExpressionType, HasExpressionType, TypeQualifier, UserDefinedTypes,
};

pub trait HasFunctions {
    fn functions(&self) -> &FunctionMap;
}

pub trait HasSubs {
    fn subs(&self) -> &SubMap;
}

pub trait HasUserDefinedTypes {
    fn user_defined_types(&self) -> &UserDefinedTypes;
}

/// Checks if a type can be cast into another type.
pub trait CanCastTo<T> {
    /// Checks if a type can be cast into another type.
    fn can_cast_to(&self, target: &T) -> bool;
}

impl CanCastTo<TypeQualifier> for TypeQualifier {
    /// Checks if this `TypeQualifier` can be cast into the given one.
    ///
    /// # Examples
    ///
    /// ```
    /// use rusty_linter::CanCastTo;
    /// use rusty_parser::specific::TypeQualifier;
    ///
    /// assert!(TypeQualifier::BangSingle.can_cast_to(&TypeQualifier::PercentInteger));
    /// assert!(TypeQualifier::DollarString.can_cast_to(&TypeQualifier::DollarString));
    /// assert!(!TypeQualifier::HashDouble.can_cast_to(&TypeQualifier::DollarString));
    /// assert!(!TypeQualifier::DollarString.can_cast_to(&TypeQualifier::AmpersandLong));
    /// ```
    fn can_cast_to(&self, other: &Self) -> bool {
        match self {
            Self::DollarString => matches!(other, Self::DollarString),
            _ => !matches!(other, Self::DollarString),
        }
    }
}

impl CanCastTo<TypeQualifier> for ExpressionType {
    fn can_cast_to(&self, other: &TypeQualifier) -> bool {
        match self {
            Self::BuiltIn(q_left) => q_left.can_cast_to(other),
            Self::FixedLengthString(_) => matches!(other, TypeQualifier::DollarString),
            _ => false,
        }
    }
}

impl CanCastTo<ExpressionType> for ExpressionType {
    fn can_cast_to(&self, other: &Self) -> bool {
        match self {
            Self::BuiltIn(q_left) => match other {
                Self::BuiltIn(q_right) => q_left.can_cast_to(q_right),
                Self::FixedLengthString(_) => matches!(q_left, TypeQualifier::DollarString),
                _ => false,
            },
            Self::FixedLengthString(_) => matches!(
                other,
                Self::BuiltIn(TypeQualifier::DollarString) | Self::FixedLengthString(_)
            ),
            Self::UserDefined(u_left) => match other {
                Self::UserDefined(u_right) => u_left == u_right,
                _ => false,
            },
            Self::Unresolved | Self::Array(_) => false,
        }
    }
}

impl CanCastTo<TypeQualifier> for Expression {
    fn can_cast_to(&self, other: &TypeQualifier) -> bool {
        self.expression_type().can_cast_to(other)
    }
}

impl CanCastTo<TypeQualifier> for ExpressionPos {
    fn can_cast_to(&self, target: &TypeQualifier) -> bool {
        self.element.can_cast_to(target)
    }
}

impl CanCastTo<Expression> for ExpressionPos {
    fn can_cast_to(&self, target: &Expression) -> bool {
        self.expression_type()
            .can_cast_to(&target.expression_type())
    }
}

impl CanCastTo<ExpressionPos> for ExpressionPos {
    fn can_cast_to(&self, target: &ExpressionPos) -> bool {
        self.expression_type()
            .can_cast_to(&target.expression_type())
    }
}

impl CanCastTo<ResolvedParamType> for ExpressionType {
    fn can_cast_to(&self, target: &ResolvedParamType) -> bool {
        match self {
            Self::Unresolved => false,
            Self::BuiltIn(q) => match target {
                ResolvedParamType::BuiltIn(q_target, _) => q.can_cast_to(q_target),
                _ => false,
            },
            Self::FixedLengthString(_) => {
                matches!(
                    target,
                    ResolvedParamType::BuiltIn(TypeQualifier::DollarString, _)
                )
            }
            Self::UserDefined(type_name) => match target {
                ResolvedParamType::UserDefined(target_type_name) => type_name == target_type_name,
                _ => false,
            },
            Self::Array(box_element_type) => match target {
                ResolvedParamType::Array(target_element_type) => {
                    box_element_type.can_cast_to(target_element_type)
                }
                _ => false,
            },
        }
    }
}

impl CanCastTo<Box<ResolvedParamType>> for Box<ExpressionType> {
    fn can_cast_to(&self, target: &Box<ResolvedParamType>) -> bool {
        self.as_ref().can_cast_to(target.as_ref())
    }
}
