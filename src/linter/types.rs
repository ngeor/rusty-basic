use crate::common::{CanCastTo, Locatable};
use crate::linter::pre_linter::{FunctionSignature, SubSignature};
use crate::parser::{BareName, BuiltInStyle, ExpressionType, QualifiedName, TypeQualifier};
use std::collections::HashMap;

pub type SubMap = HashMap<BareName, Locatable<SubSignature>>;
pub type FunctionMap = HashMap<BareName, Locatable<FunctionSignature>>;

/// Holds the resolved name of a subprogram.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SubprogramName {
    /// The resolved name of a function.
    Function(QualifiedName),

    /// The resolved name of a sub.
    Sub(BareName),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameContext {
    Global,
    Sub,
    Function,
}

#[derive(Eq, PartialEq)]
pub enum ResolvedParamType {
    BuiltIn(TypeQualifier, BuiltInStyle),
    UserDefined(BareName),
    Array(Box<ResolvedParamType>),
}

impl CanCastTo<&ResolvedParamType> for ExpressionType {
    fn can_cast_to(&self, target: &ResolvedParamType) -> bool {
        match self {
            Self::Unresolved => false,
            Self::BuiltIn(q) => match target {
                ResolvedParamType::BuiltIn(q_target, _) => q.can_cast_to(*q_target),
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
                ResolvedParamType::Array(target_element_type) => box_element_type
                    .as_ref()
                    .can_cast_to(target_element_type.as_ref()),
                _ => false,
            },
        }
    }
}
