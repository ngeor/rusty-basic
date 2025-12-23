use rusty_parser::specific::{BuiltInStyle, TypeQualifier, VariableInfo};
use rusty_variant::Variant;

use crate::names::{compacts_info::CompactsInfo, traits::SingleNameTrait};

pub struct NameInfo(NameInfoInner);

impl NameInfo {
    pub fn constant(v: Variant) -> Self {
        Self(NameInfoInner::Constant(v))
    }

    pub fn compacts() -> Self {
        Self(NameInfoInner::Compacts(CompactsInfo::default()))
    }

    pub fn extended(variable_info: VariableInfo) -> Self {
        Self(NameInfoInner::Extended(variable_info))
    }
}

impl SingleNameTrait for NameInfo {
    fn get_compact(&self, qualifier: TypeQualifier) -> Option<&VariableInfo> {
        self.0.get_compact(qualifier)
    }

    fn get_extended(&self) -> Option<&VariableInfo> {
        self.0.get_extended()
    }

    fn get_const_value(&self) -> Option<&Variant> {
        self.0.get_const_value()
    }

    fn collect_var_info(&self, only_shared: bool) -> Vec<(BuiltInStyle, &VariableInfo)> {
        self.0.collect_var_info(only_shared)
    }

    fn insert_compact(&mut self, variable_info: VariableInfo) {
        self.0.insert_compact(variable_info);
    }
}

enum NameInfoInner {
    Constant(Variant),
    Compacts(CompactsInfo),
    Extended(VariableInfo),
}

impl SingleNameTrait for NameInfoInner {
    fn get_compact(&self, qualifier: TypeQualifier) -> Option<&VariableInfo> {
        match self {
            Self::Compacts(compacts) => compacts.get_compact(qualifier),
            _ => None,
        }
    }

    fn get_extended(&self) -> Option<&VariableInfo> {
        match self {
            Self::Extended(v) => Some(v),
            _ => None,
        }
    }

    fn get_const_value(&self) -> Option<&Variant> {
        match self {
            Self::Constant(v) => Some(v),
            _ => None,
        }
    }

    fn collect_var_info(&self, only_shared: bool) -> Vec<(BuiltInStyle, &VariableInfo)> {
        match self {
            Self::Compacts(compacts) => compacts.collect_var_info(only_shared),
            Self::Extended(v) => {
                if v.shared || !only_shared {
                    vec![(BuiltInStyle::Extended, v)]
                } else {
                    vec![]
                }
            }
            Self::Constant(_) => vec![],
        }
    }

    fn insert_compact(&mut self, variable_info: VariableInfo) {
        match self {
            Self::Compacts(compacts) => compacts.insert_compact(variable_info),
            _ => panic!("Cannot insert compact because it already exists as CONST or extended"),
        }
    }
}
