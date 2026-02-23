use std::collections::HashMap;

use rusty_common::CaseInsensitiveString;
use rusty_parser::{
    AsBareName, BareName, BuiltInStyle, Name, RedimInfo, TypeQualifier, VarType, VariableInfo
};
use rusty_variant::Variant;

use crate::core::{ConstLookup, NameScope, SubprogramName};
use crate::names::ImplicitVars;
use crate::names::names_inner::NamesInner;
use crate::names::traits::ManyNamesTrait;

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
    data: HashMap<SubprogramName, NamesOneLevel>,
    current_key: SubprogramName,
}

/// Stores the data relevant to one level only (i.e. global symbols, or a FUNCTION, or a SUB).
/// Collects constant and variable names in [NamesInner] and implicit variables in [ImplicitVars].
/// TODO merge [ImplicitVars] into [NamesInner]
/// TODO use bi_tuple macro
#[derive(Default)]
struct NamesOneLevel(NamesInner, ImplicitVars);

impl Names {
    pub fn new() -> Self {
        let mut data: HashMap<SubprogramName, NamesOneLevel> = HashMap::new();
        // insert GLOBAL scope
        data.insert(SubprogramName::Global, NamesOneLevel::default());
        Self {
            data,
            current_key: SubprogramName::Global,
        }
    }

    fn current_data(&self) -> &NamesOneLevel {
        self.data.get(&self.current_key).unwrap()
    }

    fn current_data_mut(&mut self) -> &mut NamesOneLevel {
        self.data.get_mut(&self.current_key).unwrap()
    }

    pub fn get_implicit_vars_mut(&mut self) -> &mut ImplicitVars {
        &mut self.current_data_mut().1
    }

    pub fn names(&self) -> &NamesInner {
        &self.current_data().0
    }

    pub fn names_mut(&mut self) -> &mut NamesInner {
        &mut self.current_data_mut().0
    }

    /// Returns the global names, but only if we are currently within a sub program.
    fn global_names(&self) -> Option<&NamesInner> {
        match &self.current_key {
            SubprogramName::Global => None,
            _ => self.data.get(&SubprogramName::Global).map(|x| &x.0),
        }
    }

    pub fn get_compact_var_recursively(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        self.names().get_compact(bare_name, qualifier).or_else(|| {
            self.global_names().and_then(|global_names| {
                Self::require_shared(global_names.get_compact(bare_name, qualifier))
            })
        })
    }

    pub fn get_extended_var_recursively(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        self.names().get_extended(bare_name).or_else(|| {
            self.global_names()
                .and_then(|global_names| Self::require_shared(global_names.get_extended(bare_name)))
        })
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
        self.names().contains_key(bare_name)
            || self.get_extended_var_recursively(bare_name).is_some()
    }

    pub fn get_const_value_recursively(&self, bare_name: &BareName) -> Option<&Variant> {
        self.names().get_const_value(bare_name).or_else(|| {
            self.global_names()
                .and_then(|global_names| global_names.get_const_value(bare_name))
        })
    }

    pub fn contains_const_recursively(&self, bare_name: &BareName) -> bool {
        self.get_const_value_recursively(bare_name).is_some()
    }

    pub fn insert<T: VarType>(
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
            self.names_mut().insert_extended(bare_name, variable_info)
        } else {
            self.insert_compact(bare_name, variable_info)
        }
    }

    pub fn insert_compact(&mut self, bare_name: BareName, variable_info: VariableInfo) {
        self.names_mut().insert_compact(bare_name, variable_info)
    }

    pub fn is_in_function(&self, function_name: &BareName) -> bool {
        match &self.current_key {
            SubprogramName::Function(f) => f.as_bare_name() == function_name,
            _ => false,
        }
    }

    pub fn is_in_subprogram(&self) -> bool {
        self.current_key != SubprogramName::Global
    }

    pub fn get_name_scope(&self) -> NameScope {
        NameScope::from(&self.current_key)
    }

    pub fn push(&mut self, subprogram_name: SubprogramName) {
        debug_assert!(
            !self.data.contains_key(&subprogram_name),
            "should not encounter same function/sub twice!"
        );
        debug_assert_ne!(
            subprogram_name,
            SubprogramName::Global,
            "should not push global scope"
        );
        self.current_key = subprogram_name.clone();
        self.data.insert(subprogram_name, NamesOneLevel::default());
    }

    pub fn pop(&mut self) {
        debug_assert_ne!(
            self.current_key,
            SubprogramName::Global,
            "should not pop context from the global level"
        );
        self.current_key = SubprogramName::Global;
    }

    pub fn find_name_or_shared_in_parent(
        &self,
        bare_name: &BareName,
    ) -> Vec<(BuiltInStyle, &VariableInfo)> {
        let mut result = self.names().collect_var_info(bare_name, false);
        if let Some(global_names) = self.global_names() {
            result.extend(global_names.collect_var_info(bare_name, true));
        }
        result
    }

    pub fn get_resolved_variable_info(
        &self,
        subprogram_name: &SubprogramName,
        name: &Name,
    ) -> &VariableInfo {
        let one_level = self
            .data
            .get(subprogram_name)
            .unwrap_or_else(|| panic!("Subprogram {:?} should be resolved", subprogram_name));
        match one_level.0.get_variable_info_by_name(name) {
            Some(i) => i,
            None => {
                if subprogram_name != &SubprogramName::Global {
                    // try then parent level but only for SHARED
                    let result = self
                        .data
                        .get(&SubprogramName::Global)
                        .expect("Global name scope missing!")
                        .0
                        .get_variable_info_by_name(name)
                        .unwrap_or_else(|| {
                            panic!("Could not resolve {} even in global namespace", name)
                        });
                    if result.shared {
                        result
                    } else {
                        panic!("{} should be resolved", name)
                    }
                } else {
                    panic!("{} should be resolved", name)
                }
            }
        }
    }
}

impl ConstLookup for Names {
    fn get_const_value(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.names().get_const_value(name).or_else(|| {
            self.global_names()
                .and_then(|global_names| global_names.get_const_value(name))
        })
    }
}

impl Default for Names {
    fn default() -> Self {
        Self::new()
    }
}
