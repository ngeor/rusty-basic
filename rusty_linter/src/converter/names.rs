use crate::const_value_resolver::ConstLookup;
use crate::converter::types::Implicits;
use crate::NameContext;
use rusty_common::CaseInsensitiveString;
use rusty_parser::{
    BareName, BuiltInStyle, HasExpressionType, QualifiedNamePos, RedimInfo, TypeQualifier,
    VarTypeIsExtended, VariableInfo,
};
use rusty_variant::Variant;
use std::collections::HashMap;

/*

Naming rules

1. It is possible to have multiple compact variables

e.g. A = 3.14 (resolves as A! by the default rules), A$ = "hello", A% = 1

2. A constant can be referenced either bare or by its correct qualifier

2b. A constant cannot co-exist with other symbols of the same name

3. A bare constant gets its qualifier from the expression and not from the type resolver

4. A constant in a subprogram can override a global constant

5. An extended variable can be referenced either bare or by its correct qualifier
5b. An extended variable cannot co-exist with other symbols of the same name
*/

pub struct Names {
    map: HashMap<BareName, NameInfo>,
    current_function_name: Option<BareName>,
    parent: Option<Box<Names>>,
    // TODO implicits has nothing to do with Names, it's only here because of the convenience of pushing/popping a Names context
    implicits: Implicits,
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

    pub fn add_implicit(&mut self, name_pos: QualifiedNamePos) {
        self.implicits.push(name_pos);
    }

    pub fn get_implicits(&mut self) -> &mut Implicits {
        &mut self.implicits
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
            Some(NameInfo::Compacts(qualifiers)) => qualifiers.get(&qualifier),
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
            Some(NameInfo::Compacts(compacts)) => {
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
                self.map.insert(bare_name, NameInfo::Compacts(map));
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
        self.parent.map(|boxed_parent| *boxed_parent)
    }

    pub fn find_name_or_shared_in_parent(
        &self,
        bare_name: &BareName,
    ) -> Vec<(BuiltInStyle, &VariableInfo)> {
        self.find_name(bare_name, false)
    }

    fn find_name(
        &self,
        bare_name: &BareName,
        only_shared: bool,
    ) -> Vec<(BuiltInStyle, &VariableInfo)> {
        let mut result = Vec::<(BuiltInStyle, &VariableInfo)>::new();
        if let Some(name_info) = self.map.get(bare_name) {
            match name_info {
                NameInfo::Compacts(map) => {
                    result.extend(
                        map.values()
                            .filter(|v| v.shared || !only_shared)
                            .map(|v| (BuiltInStyle::Compact, v)),
                    );
                }
                NameInfo::Extended(v) => {
                    result.extend(
                        std::iter::once(v)
                            .filter(|v| v.shared || !only_shared)
                            .map(|v| (BuiltInStyle::Extended, v)),
                    );
                }
                NameInfo::Constant(_) => {
                    if !only_shared {
                        panic!("Should have detected for constants before calling this method");
                    }
                }
            }
        }
        if let Some(boxed_parent) = &self.parent {
            result.extend(boxed_parent.find_name(bare_name, true).into_iter());
        }
        result
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

enum NameInfo {
    Constant(Variant),
    Compacts(HashMap<TypeQualifier, VariableInfo>),
    Extended(VariableInfo),
}
