use crate::common::CaseInsensitiveString;
use crate::linter::const_value_resolver::ConstLookup;
use crate::linter::converter::converter::Implicits;
use crate::linter::NameContext;
use crate::parser::{
    BareName, BuiltInStyle, HasExpressionType, QualifiedNameNode, RedimInfo, TypeQualifier,
    VarTypeIsExtended, VariableInfo,
};
use crate::variant::Variant;
use std::collections::hash_map::Values;
use std::collections::{HashMap, HashSet};

pub struct Names {
    map: HashMap<BareName, NameInfo>,
    current_function_name: Option<BareName>,
    parent: Option<Box<Names>>,
    implicits: Implicits,
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
            implicits: Implicits::new(),
        }
    }

    pub fn new_root() -> Self {
        Self::new(None, None)
    }

    pub fn add_implicit(&mut self, name_node: QualifiedNameNode) {
        self.implicits.push(name_node);
    }

    pub fn get_implicits(&mut self) -> &mut Implicits {
        &mut self.implicits
    }

    pub fn visit_names<F, T, E>(&self, bare_name: &BareName, f: F) -> Result<T, E>
    where
        F: Fn(BuiltInStyle, &VariableInfo) -> Result<T, E>,
        T: Default,
    {
        for (built_in_style, variable_info) in self.names_iterator(bare_name) {
            f(built_in_style, variable_info)?;
        }
        Ok(T::default())
    }

    /// Returns true if this name is a constant, or an extended variable,
    /// or a compact variable. In the case of compact variables, multiple may
    /// exist with the same bare name, e.g. `A$` and `A%`.
    pub fn contains(&self, bare_name: &BareName) -> bool {
        self.map.contains_key(bare_name)
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

    pub fn insert<T: HasExpressionType + VarTypeIsExtended>(
        &mut self,
        bare_name: BareName,
        dim_type: &T,
        shared: bool,
        redim_info: Option<RedimInfo>,
    ) {
        let variable_info = VariableInfo {
            expression_type: dim_type.expression_type(),
            shared,
            redim_info,
        };
        if dim_type.is_extended() {
            self.insert_extended(bare_name, variable_info)
        } else {
            self.insert_compact(bare_name, variable_info)
        }
    }

    pub fn insert_const(&mut self, bare_name: BareName, v: Variant) {
        debug_assert!(!self.map.contains_key(&bare_name));
        self.map.insert(bare_name, NameInfo::Constant(v));
    }

    fn insert_compact(&mut self, bare_name: BareName, variable_info: VariableInfo) {
        let q = variable_info
            .expression_type
            .opt_qualifier()
            .expect("Should be resolved");
        match self.map.get_mut(&bare_name) {
            Some(NameInfo::Compact(compacts)) => {
                Self::insert_in_compacts(compacts, q, variable_info);
            }
            Some(_) => {
                panic!(
                    "Cannot insert compact {} because it already exists as CONST or extended",
                    bare_name
                );
            }
            None => {
                let mut map: HashMap<TypeQualifier, VariableInfo> = HashMap::new();
                Self::insert_in_compacts(&mut map, q, variable_info);
                self.map.insert(bare_name, NameInfo::Compact(map));
            }
        }
    }

    fn insert_in_compacts(
        map: &mut HashMap<TypeQualifier, VariableInfo>,
        q: TypeQualifier,
        variable_info: VariableInfo,
    ) {
        debug_assert!(match map.get(&q) {
            Some(existing_v) => {
                existing_v.redim_info.is_some()
            }
            None => {
                true
            }
        });
        map.insert(q, variable_info);
    }

    fn insert_extended(&mut self, bare_name: BareName, variable_context: VariableInfo) {
        debug_assert!(match self.map.get(&bare_name) {
            Some(NameInfo::Extended(v)) => {
                v.redim_info.is_some()
            }
            Some(_) => false,
            None => {
                true
            }
        });
        self.map
            .insert(bare_name, NameInfo::Extended(variable_context));
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

    pub fn get_name_context(&self) -> NameContext {
        if self.parent.is_some() {
            if self.current_function_name.is_some() {
                NameContext::Function
            } else {
                NameContext::Sub
            }
        } else {
            NameContext::Global
        }
    }

    pub fn pop_parent(self) -> Option<Self> {
        match self.parent {
            Some(boxed_parent) => Some(*boxed_parent),
            _ => None,
        }
    }

    pub fn names_iterator<'a>(
        &'a self,
        bare_name: &'a BareName,
    ) -> impl Iterator<Item = (BuiltInStyle, &'a VariableInfo)> {
        NamesIterator::new(self, bare_name)
    }
}

impl ConstLookup for Names {
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

struct NamesIterator<'a> {
    names: &'a Names,
    bare_name: &'a BareName,
    state: NamesIteratorState,
    compacts: Option<Values<'a, TypeQualifier, VariableInfo>>,
    only_shared: bool,
}

enum NamesIteratorState {
    NotStarted,
    Compacts,
    FinishedCurrent,
    Finished,
}

impl<'a> NamesIterator<'a> {
    pub fn new(names: &'a Names, bare_name: &'a BareName) -> Self {
        Self {
            names,
            bare_name,
            state: NamesIteratorState::NotStarted,
            compacts: None,
            only_shared: false,
        }
    }

    fn on_not_started(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.names.map.get(self.bare_name) {
            Some(NameInfo::Compact(m)) => {
                self.compacts = Some(m.values());
                self.state = NamesIteratorState::Compacts;
                self.on_compacts()
            }
            Some(NameInfo::Extended(v)) => {
                self.state = NamesIteratorState::FinishedCurrent;
                if !self.only_shared || v.shared {
                    Some((BuiltInStyle::Extended, v))
                } else {
                    self.on_finished_current()
                }
            }
            Some(NameInfo::Constant(_)) => {
                if self.only_shared {
                    self.on_finished_current()
                } else {
                    panic!("Should have detected for constants before calling this method");
                }
            }
            _ => self.on_finished_current(),
        }
    }

    fn on_compacts(&mut self) -> Option<<Self as Iterator>::Item> {
        let v = self.compacts.as_mut().unwrap().next();
        match v {
            Some(v) => {
                if v.shared || !self.only_shared {
                    Some((BuiltInStyle::Compact, v))
                } else {
                    self.on_compacts()
                }
            }
            _ => {
                self.state = NamesIteratorState::FinishedCurrent;
                self.on_finished_current()
            }
        }
    }

    fn on_finished_current(&mut self) -> Option<<Self as Iterator>::Item> {
        // go to parent, but only shared
        self.only_shared = true;
        match &self.names.parent {
            Some(parent_names) => {
                self.names = parent_names.as_ref();
                self.state = NamesIteratorState::NotStarted;
                self.on_not_started()
            }
            None => {
                self.state = NamesIteratorState::Finished;
                None
            }
        }
    }
}

impl<'a> Iterator for NamesIterator<'a> {
    type Item = (BuiltInStyle, &'a VariableInfo);

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            NamesIteratorState::NotStarted => self.on_not_started(),
            NamesIteratorState::Compacts => self.on_compacts(),
            NamesIteratorState::FinishedCurrent => self.on_finished_current(),
            NamesIteratorState::Finished => None,
        }
    }
}
