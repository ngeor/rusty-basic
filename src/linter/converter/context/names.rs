use crate::common::CaseInsensitiveString;
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::parser::{BareName, Name, TypeQualifier, VariableInfo};
use crate::variant::Variant;
use std::collections::{HashMap, HashSet};

pub struct Names {
    compact_name_set: HashMap<BareName, HashSet<TypeQualifier>>,
    compact_names: HashMap<Name, VariableInfo>,
    extended_names: HashMap<BareName, VariableInfo>,
    constants: HashMap<BareName, Variant>,
    current_function_name: Option<BareName>,
    parent: Option<Box<Names>>,
}

impl Names {
    pub fn new(parent: Option<Box<Self>>, current_function_name: Option<BareName>) -> Self {
        Self {
            compact_name_set: HashMap::new(),
            compact_names: HashMap::new(),
            extended_names: HashMap::new(),
            constants: HashMap::new(),
            current_function_name,
            parent,
        }
    }

    pub fn new_root() -> Self {
        Self::new(None, None)
    }

    pub fn contains_local_var_or_local_const(&self, bare_name: &BareName) -> bool {
        self.compact_name_set.contains_key(bare_name)
            || self.extended_names.contains_key(bare_name)
            || self.constants.contains_key(bare_name)
    }

    /// Checks if a new compact variable can be introduced for the given name and qualifier.
    /// This is allowed if the given name is not yet known, or if it is known as a compact
    /// name and the qualifier hasn't been used yet.
    pub fn can_accept_compact(&self, bare_name: &BareName, qualifier: TypeQualifier) -> bool {
        if self.extended_names.contains_key(bare_name) || self.constants.contains_key(bare_name) {
            false
        } else {
            match self.compact_name_set.get(bare_name) {
                Some(qualifiers) => !qualifiers.contains(&qualifier),
                _ => true,
            }
        }
    }

    fn get_local_compact_var(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match self.compact_name_set.get(bare_name) {
            Some(qualifiers) => {
                if qualifiers.contains(&qualifier) {
                    let name = Name::new(bare_name.clone(), Some(qualifier));
                    self.compact_names.get(&name)
                } else {
                    None
                }
            }
            _ => None,
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

    fn get_local_extended_var(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        self.extended_names.get(bare_name)
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

    fn get_extended_shared_var_recursively(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        match Self::require_shared(self.get_local_extended_var(bare_name)) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => parent_names.get_extended_shared_var_recursively(bare_name),
                _ => None,
            },
        }
    }

    pub fn contains_const(&self, bare_name: &BareName) -> bool {
        self.constants.contains_key(bare_name)
    }

    pub fn get_const_value_no_recursion(&self, bare_name: &BareName) -> Option<&Variant> {
        self.constants.get(bare_name)
    }

    pub fn get_const_value_recursively(&self, bare_name: &BareName) -> Option<&Variant> {
        match self.constants.get(bare_name) {
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
        self.compact_names
            .insert(Name::new(bare_name.clone(), Some(q)), variable_context);
        match self.compact_name_set.get_mut(&bare_name) {
            Some(s) => {
                s.insert(q);
            }
            None => {
                let mut s: HashSet<TypeQualifier> = HashSet::new();
                s.insert(q);
                self.compact_name_set.insert(bare_name, s);
            }
        }
    }

    pub fn insert_extended(&mut self, bare_name: BareName, variable_context: VariableInfo) {
        self.extended_names.insert(bare_name, variable_context);
    }

    pub fn insert_const(&mut self, bare_name: BareName, v: Variant) {
        self.constants.insert(bare_name, v);
    }

    pub fn drain_extended_names_into(&mut self, set: &mut HashSet<BareName>) {
        set.extend(self.extended_names.drain().map(|(k, _)| k));
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
        match self.constants.get(name) {
            Some(v) => Some(v),
            _ => match &self.parent {
                Some(boxed_parent) => boxed_parent.get_resolved_constant(name),
                _ => None,
            },
        }
    }
}
