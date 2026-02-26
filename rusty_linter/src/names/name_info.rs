use rusty_parser::{BuiltInStyle, TypeQualifier};
use rusty_variant::Variant;

use crate::core::VariableInfo;
use crate::names::compacts::Compacts;
use crate::names::traits::SingleNameTrait;

/// Stores information about a constant or variable name.
/// The name itself isn't stored here.
pub struct NameInfo {
    inner: NameInfoInner,
}

impl NameInfo {
    pub fn constant(v: Variant) -> Self {
        Self {
            inner: NameInfoInner::Constant { value: v },
        }
    }

    pub fn compacts() -> Self {
        Self {
            inner: NameInfoInner::Compacts {
                compacts: Compacts::default(),
            },
        }
    }

    pub fn extended(variable_info: VariableInfo) -> Self {
        Self {
            inner: NameInfoInner::Extended { variable_info },
        }
    }
}

impl SingleNameTrait for NameInfo {
    fn get_compact(&self, qualifier: TypeQualifier) -> Option<&VariableInfo> {
        self.inner.get_compact(qualifier)
    }

    fn get_extended(&self) -> Option<&VariableInfo> {
        self.inner.get_extended()
    }

    fn get_const_value(&self) -> Option<&Variant> {
        self.inner.get_const_value()
    }

    fn collect_var_info(&self, only_shared: bool) -> Vec<(BuiltInStyle, &VariableInfo)> {
        self.inner.collect_var_info(only_shared)
    }

    fn insert_compact(&mut self, variable_info: VariableInfo) {
        self.inner.insert_compact(variable_info);
    }
}

/// Nested enum for [NameInfo].
/// The external struct is preventing direct access to the enum members
/// outside of the module.
enum NameInfoInner {
    Constant { value: Variant },
    Compacts { compacts: Compacts },
    Extended { variable_info: VariableInfo },
}

impl SingleNameTrait for NameInfoInner {
    fn get_compact(&self, qualifier: TypeQualifier) -> Option<&VariableInfo> {
        match self {
            Self::Compacts { compacts } => compacts.get_compact(qualifier),
            _ => None,
        }
    }

    fn get_extended(&self) -> Option<&VariableInfo> {
        match self {
            Self::Extended { variable_info } => Some(variable_info),
            _ => None,
        }
    }

    fn get_const_value(&self) -> Option<&Variant> {
        match self {
            Self::Constant { value } => Some(value),
            _ => None,
        }
    }

    fn collect_var_info(&self, only_shared: bool) -> Vec<(BuiltInStyle, &VariableInfo)> {
        match self {
            Self::Compacts { compacts } => compacts.collect_var_info(only_shared),
            Self::Extended { variable_info } => {
                if variable_info.shared || !only_shared {
                    vec![(BuiltInStyle::Extended, variable_info)]
                } else {
                    vec![]
                }
            }
            Self::Constant { .. } => vec![],
        }
    }

    fn insert_compact(&mut self, variable_info: VariableInfo) {
        match self {
            Self::Compacts { compacts } => compacts.insert_compact(variable_info),
            _ => panic!("Cannot insert compact because it already exists as CONST or extended"),
        }
    }
}
