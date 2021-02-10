use crate::common::CaseInsensitiveString;
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::parser::{BareName, TypeQualifier, VariableInfo};
use crate::variant::Variant;
use std::collections::{HashMap, HashSet};

pub struct Names {
    map: HashMap<BareName, NameInfo>,
    current_function_name: Option<BareName>,
    parent: Option<Box<Names>>,
}

pub enum NameInfo {
    Constant(Variant),
    Compact(HashMap<TypeQualifier, VariableInfo>),
    Extended(VariableInfo),
}

impl Names {
    pub fn new(parent: Option<Box<Self>>, current_function_name: Option<BareName>) -> Self {
        Self {
            map: HashMap::new(),
            current_function_name,
            parent,
        }
    }

    pub fn new_root() -> Self {
        Self::new(None, None)
    }

    /// Returns true if this name is a constant, or an extended variable,
    /// or a compact variable. In the case of compact variables, multiple may
    /// exist with the same bare name, e.g. `A$` and `A%`.
    pub fn contains(&self, bare_name: &BareName) -> bool {
        self.map.contains_key(bare_name)
    }

    /// Checks if a new compact variable can be introduced for the given name and qualifier.
    /// This is allowed if the given name is not yet known, or if it is known as a compact
    /// name and the qualifier hasn't been used yet.
    pub fn can_insert_compact(&self, bare_name: &BareName, qualifier: TypeQualifier) -> bool {
        match self.map.get(bare_name) {
            Some(NameInfo::Compact(qualifiers)) => !qualifiers.contains_key(&qualifier),
            Some(_) => false,
            _ => true,
        }
    }

    pub fn get_compact_var_recursively(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match self.get_local_compact_var(bare_name, qualifier) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => {
                    parent_names.get_compact_shared_var_recursively(bare_name, qualifier)
                }
                _ => None,
            },
        }
    }

    fn get_local_compact_var(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match self.map.get(bare_name) {
            Some(NameInfo::Compact(qualifiers)) => qualifiers.get(&qualifier),
            _ => None,
        }
    }

    fn get_compact_shared_var_recursively(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match Self::require_shared(self.get_local_compact_var(bare_name, qualifier)) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => {
                    parent_names.get_compact_shared_var_recursively(bare_name, qualifier)
                }
                _ => None,
            },
        }
    }

    pub fn get_extended_var_recursively(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        match self.get_local_extended_var(bare_name) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => parent_names.get_extended_shared_var_recursively(bare_name),
                _ => None,
            },
        }
    }

    fn get_local_extended_var(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        match self.map.get(bare_name) {
            Some(NameInfo::Extended(variable_info)) => Some(variable_info),
            _ => None,
        }
    }

    fn get_extended_shared_var_recursively(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        match Self::require_shared(self.get_local_extended_var(bare_name)) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => parent_names.get_extended_shared_var_recursively(bare_name),
                _ => None,
            },
        }
    }

    fn require_shared(opt_variable_info: Option<&VariableInfo>) -> Option<&VariableInfo> {
        match opt_variable_info {
            Some(variable_info) => {
                if variable_info.shared {
                    opt_variable_info
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn contains_any_locally_or_contains_extended_recursively(
        &self,
        bare_name: &BareName,
    ) -> bool {
        self.contains(bare_name) || self.get_extended_var_recursively(bare_name).is_some()
    }

    pub fn contains_const(&self, bare_name: &BareName) -> bool {
        self.get_const_value_no_recursion(bare_name).is_some()
    }

    pub fn get_const_value_no_recursion(&self, bare_name: &BareName) -> Option<&Variant> {
        match self.map.get(bare_name) {
            Some(NameInfo::Constant(v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_const_value_recursively(&self, bare_name: &BareName) -> Option<&Variant> {
        match self.get_const_value_no_recursion(bare_name) {
            Some(v) => Some(v),
            _ => {
                if let Some(boxed_parent) = &self.parent {
                    boxed_parent.get_const_value_recursively(bare_name)
                } else {
                    None
                }
            }
        }
    }

    pub fn contains_const_recursively(&self, bare_name: &BareName) -> bool {
        if self.contains_const(bare_name) {
            true
        } else if let Some(boxed_parent) = &self.parent {
            boxed_parent.contains_const_recursively(bare_name)
        } else {
            false
        }
    }

    pub fn insert_compact(
        &mut self,
        bare_name: BareName,
        q: TypeQualifier,
        variable_context: VariableInfo,
    ) {
        match self.map.get_mut(&bare_name) {
            Some(NameInfo::Compact(qualifiers)) => {
                qualifiers.insert(q, variable_context);
            }
            None => {
                let mut qualifiers: HashMap<TypeQualifier, VariableInfo> = HashMap::new();
                qualifiers.insert(q, variable_context);
                self.map.insert(bare_name, NameInfo::Compact(qualifiers));
            }
            _ => {
                panic!("Cannot add compact")
            }
        }
    }

    pub fn insert_extended(&mut self, bare_name: BareName, variable_context: VariableInfo) {
        self.map
            .insert(bare_name, NameInfo::Extended(variable_context));
    }

    pub fn insert_const(&mut self, bare_name: BareName, v: Variant) {
        self.map.insert(bare_name, NameInfo::Constant(v));
    }

    pub fn drain_extended_names_into(&mut self, set: &mut HashSet<BareName>) {
        set.extend(
            self.map
                .drain()
                .filter(|(_, v)| {
                    if let NameInfo::Extended(_) = v {
                        true
                    } else {
                        false
                    }
                })
                .map(|(k, _)| k),
        );
    }

    pub fn is_in_function(&self, function_name: &BareName) -> bool {
        match &self.current_function_name {
            Some(f) => f == function_name,
            _ => false,
        }
    }

    pub fn is_in_subprogram(&self) -> bool {
        self.parent.is_some()
    }

    pub fn pop_parent(self) -> Option<Self> {
        match self.parent {
            Some(boxed_parent) => Some(*boxed_parent),
            _ => None,
        }
    }
}

impl ConstValueResolver for Names {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        match self.get_const_value_no_recursion(name) {
            Some(v) => Some(v),
            _ => match &self.parent {
                Some(boxed_parent) => boxed_parent.get_resolved_constant(name),
                _ => None,
            },
        }
    }
}
